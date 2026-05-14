use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use log::{info, error, warn};
use anyhow::Result;
use crate::MAX_DEVICES;

pub type ClientMap = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;

pub struct SecureServer {
    pub clients: ClientMap,
    max_clients: usize,
}

impl SecureServer {
    pub fn new(max_clients: usize) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            max_clients,
        }
    }
    
    pub async fn start<F, Fut>(&self, addr: &str, message_handler: F) -> Result<()>
    where
        F: FnMut(String, String) -> Fut + Send + Clone + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let listener = TcpListener::bind(addr).await?;
        info!("Drone server listening on {} (max {} devices)", addr, self.max_clients);
        
        while let Ok((stream, _)) = listener.accept().await {
            let clients = self.clients.clone();
            let max_clients = self.max_clients;
            let mut handler = message_handler.clone();
            
            tokio::spawn(async move {
                // Check client limit
                let client_count = clients.lock().await.len();
                if client_count >= max_clients {
                    warn!("Max devices reached ({}/{}), rejecting connection", client_count, max_clients);
                    return;
                }
                
                if let Err(e) = Self::handle_connection(stream, clients, &mut handler).await {
                    error!("Connection error: {}", e);
                }
            });
        }
        
        Ok(())
    }
    
    async fn handle_connection<F, Fut>(stream: TcpStream, clients: ClientMap, 
                                       message_handler: &mut F) -> Result<()>
    where
        F: FnMut(String, String) -> Fut + Send,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let ws_stream = accept_async(stream).await?;
        let (mut sender, mut receiver) = ws_stream.split();
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut client_id = String::new();
        let mut client_name = String::new();
        
        // Wait for client identification
        if let Some(Ok(Message::Text(msg))) = receiver.next().await {
            if let Ok(init_msg) = serde_json::from_str::<serde_json::Value>(&msg) {
                if let Some(id) = init_msg.get("client_id").and_then(|v| v.as_str()) {
                    client_id = id.to_string();
                    client_name = init_msg.get("client_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    {
                        let mut clients_lock = clients.lock().await;
                        clients_lock.insert(client_id.clone(), tx.clone());
                    }
                    
                    info!("✅ Device '{}' ({}) connected to drone", client_name, client_id);
                    
                    // Spawn task to send messages to this client
                    let send_task = tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if sender.send(msg).await.is_err() {
                                break;
                            }
                        }
                    });
                    
                    // Handle incoming messages
                    while let Some(Ok(msg)) = receiver.next().await {
                        if let Message::Text(text) = msg {
                            let text_clone = text.clone();
                            if let Err(e) = message_handler(client_id.clone(), text).await {
                                error!("Message handler error: {}. Raw text: {}", e, text_clone);
                            }
                        }
                    }
                    
                    send_task.abort();
                    {
                        let mut clients_lock = clients.lock().await;
                        clients_lock.remove(&client_id);
                    }
                    info!("Device '{}' ({}) disconnected", client_name, client_id);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn send_to_client(&self, client_id: &str, message: &str) -> Result<bool> {
        let clients = self.clients.lock().await;
        if let Some(tx) = clients.get(client_id) {
            let _ = tx.send(Message::Text(message.to_string()));
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn broadcast(&self, message: &str, exclude: Option<&str>) -> Result<usize> {
        let clients = self.clients.lock().await;
        let mut count = 0;
        
        for (id, tx) in clients.iter() {
            if Some(id.as_str()) != exclude {
                if tx.send(Message::Text(message.to_string())).is_ok() {
                    count += 1;
                }
            }
        }
        
        Ok(count)
    }
    
    pub async fn get_connected_clients(&self) -> Vec<String> {
        let clients = self.clients.lock().await;
        clients.keys().cloned().collect()
    }
}
