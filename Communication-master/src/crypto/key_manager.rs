use super::{KyberKem, DilithiumSignature};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub kyber_public: Vec<u8>,
    pub dilithium_verification: Vec<u8>,
    pub connected: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStore {
    pub kyber_public: Vec<u8>,
    pub kyber_secret: Vec<u8>,
    pub dilithium_signing: Vec<u8>,
    pub dilithium_verification: Vec<u8>,
    pub devices: HashMap<String, DeviceInfo>,
}

#[derive(Clone)]
pub struct KeyManager {
    entity_name: String,
    kyber: KyberKem,
    dilithium: DilithiumSignature,
    keystore: KeyStore,
}

impl KeyManager {
    pub fn new(entity_name: &str) -> Self {
        Self {
            entity_name: entity_name.to_string(),
            kyber: KyberKem::new(),
            dilithium: DilithiumSignature::new(),
            keystore: KeyStore {
                kyber_public: Vec::new(),
                kyber_secret: Vec::new(),
                dilithium_signing: Vec::new(),
                dilithium_verification: Vec::new(),
                devices: HashMap::new(),
            },
        }
    }
    
    pub fn generate_keys(&mut self) -> Result<()> {
        let (kyber_pub, kyber_sec) = self.kyber.generate_keypair()?;
        let (dilithium_signing, dilithium_ver) = self.dilithium.generate_keypair()?;
        
        self.keystore.kyber_public = kyber_pub;
        self.keystore.kyber_secret = kyber_sec;
        self.keystore.dilithium_signing = dilithium_signing;
        self.keystore.dilithium_verification = dilithium_ver;
        
        self.save_keys()?;
        Ok(())
    }
    
    fn get_keys_path(&self) -> PathBuf {
        PathBuf::from(format!("keys/{}.json", self.entity_name))
    }
    
    pub fn save_keys(&self) -> Result<()> {
        let path = self.get_keys_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.keystore)?;
        fs::write(path, data)?;
        Ok(())
    }
    
    pub fn load_keys(&mut self) -> Result<bool> {
        let path = self.get_keys_path();
        if path.exists() {
            let data = fs::read_to_string(path)?;
            self.keystore = serde_json::from_str(&data)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn add_device(&mut self, device_id: &str, name: &str, kyber_public: Vec<u8>, dilithium_verification: Vec<u8>) {
        self.keystore.devices.insert(device_id.to_string(), DeviceInfo {
            kyber_public,
            dilithium_verification,
            connected: true,
            name: name.to_string(),
        });
        let _ = self.save_keys();
    }
    
    pub fn remove_device(&mut self, device_id: &str) {
        self.keystore.devices.remove(device_id);
        let _ = self.save_keys();
    }
    
    pub fn get_device(&self, device_id: &str) -> Option<&DeviceInfo> {
        self.keystore.devices.get(device_id)
    }
    
    pub fn get_all_devices(&self) -> Vec<(String, DeviceInfo)> {
        self.keystore.devices.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    
    pub fn get_kyber_public(&self) -> &[u8] {
        &self.keystore.kyber_public
    }
    
    pub fn get_kyber_secret(&self) -> &[u8] {
        &self.keystore.kyber_secret
    }
    
    pub fn get_dilithium_signing(&self) -> &[u8] {
        &self.keystore.dilithium_signing
    }
    
    pub fn get_dilithium_verification(&self) -> &[u8] {
        &self.keystore.dilithium_verification
    }
    
    pub fn get_public_keys_for_exchange(&self) -> HashMap<String, String> {
        let mut keys = HashMap::new();
        keys.insert("kyber".to_string(), hex::encode(&self.keystore.kyber_public));
        keys.insert("dilithium".to_string(), hex::encode(&self.keystore.dilithium_verification));
        keys
    }
    
    pub fn device_count(&self) -> usize {
        self.keystore.devices.len()
    }
}
