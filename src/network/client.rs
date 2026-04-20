use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use anyhow::Result;
use url::Url;

pub struct SecureClient;

impl SecureClient {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn connect(&self, addr: &str, client_id: &str, client_name: &str) -> Result<(
        futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
        futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    )> {
        let url = Url::parse(&format!("ws://{}", addr))?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut sender, receiver) = ws_stream.split();
        
        // Send initialization message
        let init_msg = serde_json::json!({
            "client_id": client_id,
            "client_name": client_name,
            "type": "init"
        }).to_string();
        
        sender.send(Message::Text(init_msg)).await?;
        
        Ok((sender, receiver))
    }
}

impl Default for SecureClient {
    fn default() -> Self {
        Self::new()
    }
}
