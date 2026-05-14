use crate::crypto::{KeyManager, KyberKem, DilithiumSignature};
use crate::network::{SecureClient, MessageProtocol, MessageType};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use anyhow::Result;
use colored::*;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::io::Write;

pub struct ClientDevice {
    pub device_id: String,
    pub device_name: String,
    key_manager: KeyManager,
    kyber: KyberKem,
    dilithium: DilithiumSignature,
    client: SecureClient,
    connected_devices: Vec<(String, String)>, // (device_id, device_name)
}

impl ClientDevice {
    pub fn new(device_id: &str, device_name: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            key_manager: KeyManager::new(device_id),
            kyber: KyberKem::new(),
            dilithium: DilithiumSignature::new(),
            client: SecureClient::new(),
            connected_devices: Vec::new(),
        }
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.key_manager.load_keys()? {
            info!("Generating new keys for {}...", self.device_name);
            self.key_manager.generate_keys()?;
        } else {
            info!("Keys loaded for {}", self.device_name);
        }
        
        info!("{}", format!("📱 DEVICE '{}' INITIALIZED", self.device_name.to_uppercase()).bright_blue());
        info!("Device ID: {}", self.device_id);
        Ok(())
    }
    
    pub async fn connect_to_drone(&mut self, drone_addr: &str) -> Result<()> {
        info!("Connecting to drone at {}...", drone_addr);
        
        let (mut sender, mut receiver) = self.client.connect(drone_addr, &self.device_id, &self.device_name).await?;
        
        info!("{}", format!("✅ Connected to central drone as '{}'", self.device_name).green());
        
        // Perform key exchange
        let mut our_keys = self.key_manager.get_public_keys_for_exchange();
        our_keys.insert("name".to_string(), self.device_name.clone());
        let keys_json = serde_json::to_string(&our_keys)?;
        let key_msg = MessageProtocol::create_key_exchange(&self.device_id, "drone", &keys_json)?;
        sender.send(tokio_tungstenite::tungstenite::Message::Text(key_msg)).await?;
        
        // Handle incoming messages
        let kyber = self.kyber.clone();
        let dilithium = self.dilithium.clone();
        let mut km = self.key_manager.clone();
        let device_name = self.device_name.clone();
        let device_id = self.device_id.clone();
        
        let receive_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        if let Ok(secure_msg) = MessageProtocol::decode_message(&text) {
                            match secure_msg.msg_type {
                                MessageType::Text => {
                                    if let (Some(payload_hex), Some(signature_hex)) = (secure_msg.payload, secure_msg.signature) {
                                        if let Ok(encrypted) = hex::decode(&payload_hex) {
                                            if let Ok(signature) = hex::decode(&signature_hex) {
                                                if let Some(peer) = km.get_device(&secure_msg.sender) {
                                                    if let Ok(valid) = dilithium.verify(&encrypted, &signature, &peer.dilithium_verification) {
                                                        if valid {
                                                            if let Ok(decrypted) = kyber.decrypt_message(&encrypted, km.get_kyber_secret()) {
                                                                println!("\n{}", format!("📨 [{}] {}: {}", 
                                                                    peer.name, secure_msg.sender, decrypted).bright_yellow());
                                                                print!("> ");
                                                                let _ = std::io::stdout().flush();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                MessageType::DeviceList => {
                                    if let Some(payload) = secure_msg.payload {
                                        // payload is hex-encoded JSON of the device list
                                        if let Ok(raw_bytes) = hex::decode(&payload) {
                                            if let Ok(json_str) = String::from_utf8(raw_bytes) {
                                                if let Ok(devices) = serde_json::from_str::<Vec<(String, crate::crypto::key_manager::DeviceInfo)>>(&json_str) {
                                                    println!("\n{}", "📡 Connected devices:".cyan());
                                                    for (id, device) in devices {
                                                        println!("  - {} ({})", device.name, id);
                                                    }
                                                    print!("> ");
                                                    let _ = std::io::stdout().flush();
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        
        // Handle user input
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        
        println!("\n{}", "=".repeat(60).bright_blue());
        println!("{}", format!("💬 DEVICE '{}' - Connected to Drone Mesh", self.device_name).bold());
        println!("{}", "Commands:".cyan());
        println!("  @device_id message - Send private message to specific device");
        println!("  @all message       - Broadcast message to all devices");
        println!("  /list              - List connected devices");
        println!("  /quit              - Disconnect");
        println!("{}", "=".repeat(60).bright_blue());
        print!("> ");
        let _ = std::io::stdout().flush();
        
        while let Ok(Some(line)) = reader.next_line().await {
            let line = line.trim();
            
            if line == "/quit" {
                // Send disconnect message
                let disconnect_msg = MessageProtocol::encode_message(
                    MessageType::Disconnect, &self.device_id, "drone", None, None
                )?;
                let _ = sender.send(tokio_tungstenite::tungstenite::Message::Text(disconnect_msg)).await;
                break;
            } else if line == "/list" {
                println!("{}", "Requesting device list...".cyan());
                let list_msg = MessageProtocol::encode_message(
                    MessageType::DeviceList, &self.device_id, "drone", None, None
                )?;
                let _ = sender.send(tokio_tungstenite::tungstenite::Message::Text(list_msg)).await;
            } else if line.starts_with("@all") {
                let message = line[4..].trim();
                if !message.is_empty() {
                    // Broadcast to all devices
                    if let Some(drone_peer) = self.key_manager.get_device("drone_central") {
                        let encrypted = self.kyber.encrypt_message(message, &drone_peer.kyber_public)?;
                        let signature = self.dilithium.sign(&encrypted, self.key_manager.get_dilithium_signing())?;
                        
                        let broadcast_msg = MessageProtocol::create_broadcast(&self.device_id, encrypted, signature)?;
                        sender.send(tokio_tungstenite::tungstenite::Message::Text(broadcast_msg)).await?;
                        
                        println!("{}", format!("📢 Broadcast: {}", message).bright_green());
                    }
                }
            } else if line.starts_with("@") {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let target_id = parts[0][1..].trim();
                    let message = parts[1].trim();
                    
                    if let Some(target_device) = self.key_manager.get_device(target_id) {
                        let encrypted = self.kyber.encrypt_message(message, &target_device.kyber_public)?;
                        let signature = self.dilithium.sign(&encrypted, self.key_manager.get_dilithium_signing())?;
                        
                        let msg = MessageProtocol::create_text_message(&self.device_id, target_id, encrypted, signature)?;
                        sender.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await?;
                        
                        println!("{}", format!("📤 To {}: {}", target_id, message).bright_green());
                    } else {
                        println!("{}", format!("❌ Device '{}' not found", target_id).red());
                        println!("Use /list to see connected devices");
                    }
                }
            } else if !line.is_empty() {
                println!("{}", "⚠️ Use @device_id message or @all message".yellow());
            }
            
            print!("> ");
            let _ = std::io::stdout().flush();
        }
        
        receive_task.abort();
        info!("Disconnected from drone mesh");
        Ok(())
    }
}
