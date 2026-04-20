use crate::crypto::{KeyManager, KyberKem, DilithiumSignature};
use crate::network::{SecureServer, MessageProtocol, MessageType};
use crate::MAX_DEVICES;
use log::{info, warn, error};
use anyhow::Result;
use colored::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CentralDrone {
    key_manager: Arc<Mutex<KeyManager>>,
    kyber: KyberKem,
    dilithium: DilithiumSignature,
    server: Arc<SecureServer>,
}

impl CentralDrone {
    pub fn new() -> Self {
        Self {
            key_manager: Arc::new(Mutex::new(KeyManager::new("drone_central"))),
            kyber: KyberKem::new(),
            dilithium: DilithiumSignature::new(),
            server: Arc::new(SecureServer::new(MAX_DEVICES)),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        let mut km = self.key_manager.lock().await;
        if !km.load_keys()? {
            info!("Generating new keys for central drone...");
            km.generate_keys()?;
        } else {
            info!("Drone keys loaded");
        }
        
        info!("{}", "🚁 CENTRAL DRONE (RASPBERRY PI) INITIALIZED".bright_cyan());
        info!("Kyber Public Key: {}...", hex::encode(&km.get_kyber_public()[..32]));
        info!("Max Devices: {}", MAX_DEVICES);
        Ok(())
    }
    
    pub async fn start(&mut self, addr: &str) -> Result<()> {
        let key_manager = self.key_manager.clone();
        let kyber = self.kyber.clone();
        let dilithium = self.dilithium.clone();
        let server = self.server.clone();
        
        let handler = move |client_id: String, message: String| {
            let key_manager = key_manager.clone();
            let kyber = kyber.clone();
            let dilithium = dilithium.clone();
            let server = server.clone();
            
            async move {
                Self::handle_message(
                    client_id, message, 
                    key_manager, kyber, dilithium, 
                    server
                ).await
            }
        };
        
        info!("{}", format!("📡 Drone listening on {}", addr).green());
        info!("{}", "🔄 Drone will relay messages between all connected devices".yellow());
        
        // Print device info periodically
        self.print_device_status().await;
        
        self.server.start(addr, handler).await?;
        Ok(())
    }
    
    async fn print_device_status(&self) {
        let km = self.key_manager.lock().await;
        let device_count = km.device_count();
        info!("Connected devices: {}/{}", device_count, MAX_DEVICES);
        
        for (id, device) in km.get_all_devices() {
            info!("  - {} ({})", device.name, id);
        }
    }
    
    async fn handle_message(
        client_id: String,
        message: String,
        key_manager: Arc<Mutex<KeyManager>>,
        kyber: KyberKem,
        dilithium: DilithiumSignature,
        server: Arc<SecureServer>,
    ) -> Result<()> {
        let secure_msg = MessageProtocol::decode_message(&message)?;
        
        match secure_msg.msg_type {
            MessageType::Init => {
                info!("📱 New device connected: {}", client_id);
            }
            
            MessageType::KeyExchange => {
                Self::handle_key_exchange(&client_id, &secure_msg, key_manager.clone()).await?;
                
                // Send device list to all connected devices
                let devices_json = {
                    let km = key_manager.lock().await;
                    serde_json::to_string(&km.get_all_devices())?
                };
                let device_msg = MessageProtocol::create_device_list("drone", &devices_json)?;
                
                server.broadcast(&device_msg, Some(&client_id)).await?;
            }
            
            MessageType::Text => {
                Self::handle_text_message(&client_id, &secure_msg, key_manager, kyber, dilithium, server).await?;
            }
            
            MessageType::Broadcast => {
                Self::handle_broadcast_message(&client_id, &secure_msg, key_manager, kyber, dilithium, server).await?;
            }
            
            MessageType::Heartbeat => {
                let response = MessageProtocol::encode_message(
                    MessageType::HeartbeatAck, "drone", &client_id, None, None
                )?;
                server.send_to_client(&client_id, &response).await?;
            }
            
            MessageType::Disconnect => {
                let mut km = key_manager.lock().await;
                km.remove_device(&client_id);
                info!("Device {} disconnected", client_id);
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_key_exchange(
        client_id: &str,
        msg: &crate::network::SecureMessage,
        key_manager: Arc<Mutex<KeyManager>>,
    ) -> Result<()> {
        if let Some(payload) = &msg.payload {
            let raw_bytes = hex::decode(payload)?;
            let json_str = String::from_utf8(raw_bytes)?;
            let peer_keys: serde_json::Value = serde_json::from_str(&json_str)?;
            let kyber_pub = hex::decode(peer_keys["kyber"].as_str().unwrap())?;
            let dilithium_pub = hex::decode(peer_keys["dilithium"].as_str().unwrap())?;
            let device_name = peer_keys["name"].as_str().unwrap_or("Unknown");
            
            let mut km = key_manager.lock().await;
            km.add_device(client_id, device_name, kyber_pub, dilithium_pub);
            
            info!("{}", format!("🔐 Key exchange completed with device: {} ({})", device_name, client_id).green());
        }
        Ok(())
    }
    
    async fn handle_text_message(
        client_id: &str,
        msg: &crate::network::SecureMessage,
        key_manager: Arc<Mutex<KeyManager>>,
        kyber: KyberKem,
        dilithium: DilithiumSignature,
        server: Arc<SecureServer>,
    ) -> Result<()> {
        if let (Some(payload_hex), Some(signature_hex)) = (&msg.payload, &msg.signature) {
            let encrypted = hex::decode(payload_hex)?;
            let signature = hex::decode(signature_hex)?;
            
            let km = key_manager.lock().await;
            
            // Verify signature
            if let Some(device) = km.get_device(client_id) {
                let valid = dilithium.verify(&encrypted, &signature, &device.dilithium_verification)?;
                if !valid {
                    warn!("⚠️ Invalid signature from {}", client_id);
                    return Ok(());
                }
                
                // Decrypt message
                match kyber.decrypt_message(&encrypted, &device.kyber_public) {
                    Ok(decrypted) => {
                        info!("📨 From {} ({}): {}", device.name, client_id, decrypted);
                        
                        // Relay to specific recipient if specified
                        let target_id = &msg.receiver;
                        if target_id != "drone" && target_id != client_id {
                            if let Some(target_device) = km.get_device(target_id) {
                                // Re-encrypt for target
                                let re_encrypted = kyber.encrypt_message(&decrypted, &target_device.kyber_public)?;
                                let re_signature = dilithium.sign(&re_encrypted, km.get_dilithium_signing())?;
                                
                                let relay_msg = MessageProtocol::create_text_message(
                                    client_id, target_id, re_encrypted, re_signature
                                )?;
                                
                                if server.send_to_client(target_id, &relay_msg).await? {
                                    info!("{}", format!("🔄 Relayed message from {} to {}", 
                                           device.name, target_device.name).yellow());
                                } else {
                                    warn!("Target device {} not connected", target_id);
                                }
                            } else {
                                warn!("Unknown target device: {}", target_id);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Decryption failed: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn handle_broadcast_message(
        client_id: &str,
        msg: &crate::network::SecureMessage,
        key_manager: Arc<Mutex<KeyManager>>,
        kyber: KyberKem,
        dilithium: DilithiumSignature,
        server: Arc<SecureServer>,
    ) -> Result<()> {
        if let (Some(payload_hex), Some(signature_hex)) = (&msg.payload, &msg.signature) {
            let encrypted = hex::decode(payload_hex)?;
            let signature = hex::decode(signature_hex)?;
            
            let km = key_manager.lock().await;
            
            if let Some(device) = km.get_device(client_id) {
                let valid = dilithium.verify(&encrypted, &signature, &device.dilithium_verification)?;
                if !valid {
                    warn!("⚠️ Invalid broadcast signature from {}", client_id);
                    return Ok(());
                }
                
                match kyber.decrypt_message(&encrypted, &device.kyber_public) {
                    Ok(decrypted) => {
                        info!("📢 Broadcast from {} ({}): {}", device.name, client_id, decrypted);
                        
                        // Broadcast to all other devices
                        let devices = km.get_all_devices();
                        let total = devices.len();
                        for (target_id, target_device) in devices {
                            if target_id != client_id {
                                let re_encrypted = kyber.encrypt_message(&decrypted, &target_device.kyber_public)?;
                                let re_signature = dilithium.sign(&re_encrypted, km.get_dilithium_signing())?;
                                
                                let broadcast_msg = MessageProtocol::create_text_message(
                                    client_id, &target_id, re_encrypted, re_signature
                                )?;
                                
                                let _ = server.send_to_client(&target_id, &broadcast_msg).await;
                            }
                        }
                        
                        info!("{}", format!("📡 Broadcast sent to {} devices", total.saturating_sub(1)).cyan());
                    }
                    Err(e) => {
                        error!("Broadcast decryption failed: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for CentralDrone {
    fn default() -> Self {
        Self::new()
    }
}
