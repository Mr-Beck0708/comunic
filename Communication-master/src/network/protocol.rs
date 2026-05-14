use serde::{Serialize, Deserialize};
use chrono::Utc;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Init,
    KeyExchange,
    Text,
    Broadcast,
    DeviceList,
    Heartbeat,
    HeartbeatAck,
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMessage {
    pub msg_type: MessageType,
    pub sender: String,
    pub receiver: String,
    pub timestamp: String,
    pub payload: Option<String>,
    pub signature: Option<String>,
}

pub struct MessageProtocol;

impl MessageProtocol {
    pub fn encode_message(msg_type: MessageType, sender: &str, receiver: &str, 
                          payload: Option<Vec<u8>>, signature: Option<Vec<u8>>) -> Result<String> {
        let message = SecureMessage {
            msg_type,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            payload: payload.map(|p| hex::encode(p)),
            signature: signature.map(|s| hex::encode(s)),
        };
        
        Ok(serde_json::to_string(&message)?)
    }
    
    pub fn decode_message(data: &str) -> Result<SecureMessage> {
        let message: SecureMessage = serde_json::from_str(data)?;
        Ok(message)
    }
    
    pub fn create_text_message(sender: &str, receiver: &str, 
                               encrypted_content: Vec<u8>, signature: Vec<u8>) -> Result<String> {
        Self::encode_message(MessageType::Text, sender, receiver, 
                            Some(encrypted_content), Some(signature))
    }
    
    pub fn create_broadcast(sender: &str, encrypted_content: Vec<u8>, signature: Vec<u8>) -> Result<String> {
        Self::encode_message(MessageType::Broadcast, sender, "all", 
                            Some(encrypted_content), Some(signature))
    }
    
    pub fn create_key_exchange(sender: &str, receiver: &str, public_keys: &str) -> Result<String> {
        Self::encode_message(MessageType::KeyExchange, sender, receiver,
                            Some(public_keys.as_bytes().to_vec()), None)
    }
    
    pub fn create_device_list(sender: &str, devices: &str) -> Result<String> {
        Self::encode_message(MessageType::DeviceList, sender, "all",
                            Some(devices.as_bytes().to_vec()), None)
    }
}
