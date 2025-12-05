use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;

use super::types::*;

type PendingRequests = Arc<RwLock<HashMap<String, mpsc::Sender<WhatsAppResponse>>>>;

/// WhatsApp client that communicates with Node.js service via WebSocket
pub struct WhatsAppClient {
    write_tx: mpsc::Sender<WsMessage>,
    pending: PendingRequests,
}

impl WhatsAppClient {
    /// Connect to WhatsApp service
    pub async fn connect(url: &str, event_tx: mpsc::Sender<WhatsAppEvent>) -> Result<Self> {
        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect to WhatsApp service")?;
        
        log::info!("Connected to WhatsApp service");
        
        let (mut write, mut read) = ws_stream.split();
        let (write_tx, mut write_rx) = mpsc::channel::<WsMessage>(100);
        
        let pending: PendingRequests = Arc::new(RwLock::new(HashMap::new()));
        let pending_clone = Arc::clone(&pending);
        
        // Spawn task to write messages
        tokio::spawn(async move {
            while let Some(msg) = write_rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    log::error!("Failed to send message to WhatsApp service: {}", e);
                    break;
                }
            }
        });
        
        // Spawn task to read messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        // Try to parse as event first
                        if let Ok(event) = serde_json::from_str::<WhatsAppEvent>(&text) {
                            if let Err(e) = event_tx.send(event).await {
                                log::error!("Failed to send event to app: {}", e);
                            }
                        }
                        // Otherwise try as response
                        else if let Ok(response) = serde_json::from_str::<WhatsAppResponse>(&text) {
                            let pending = pending_clone.read().await;
                            if let Some(tx) = pending.get(&response.id) {
                                let _ = tx.send(response).await;
                            }
                        } else {
                            log::warn!("Received unknown message format: {}", text);
                        }
                    }
                    Ok(WsMessage::Close(_)) => {
                        log::warn!("WhatsApp service closed connection");
                        let _ = event_tx.send(WhatsAppEvent::Disconnected).await;
                        break;
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        let _ = event_tx.send(WhatsAppEvent::Disconnected).await;
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(Self {
            write_tx,
            pending,
        })
    }
    
    /// Send a request and wait for response
    async fn request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        self.request_with_timeout(method, params, 30).await
    }
    
    /// Send a request with custom timeout
    async fn request_with_timeout(&self, method: &str, params: serde_json::Value, timeout_secs: u64) -> Result<serde_json::Value> {
        let id = Uuid::new_v4().to_string();
        let (response_tx, mut response_rx) = mpsc::channel(1);
        
        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(id.clone(), response_tx);
        }
        
        // Send request
        let req = WhatsAppRequest {
            id: id.clone(),
            method: method.to_string(),
            params,
        };
        
        let msg = WsMessage::Text(serde_json::to_string(&req)?);
        self.write_tx.send(msg).await
            .context("Failed to send request")?;
        
        // Wait for response with custom timeout
        let response = tokio::time::timeout(
            tokio::time::Duration::from_secs(timeout_secs),
            response_rx.recv()
        )
        .await
        .context(format!("Request timed out after {}s", timeout_secs))?
        .ok_or_else(|| anyhow!("Response channel closed"))?;
        
        // Clean up pending request
        {
            let mut pending = self.pending.write().await;
            pending.remove(&id);
        }
        
        // Check for error
        if let Some(error) = response.error {
            return Err(anyhow!("WhatsApp service error: {}", error));
        }
        
        response.result.ok_or_else(|| anyhow!("No result in response"))
    }
    
    /// Get all chats (with longer timeout as this can be slow with many chats)
    pub async fn get_chats(&self) -> Result<Vec<Chat>> {
        let result = self.request_with_timeout("getChats", json!({}), 120).await?;
        
        // Log the actual JSON for debugging
        log::debug!("getChats result: {}", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "<invalid json>".to_string()));
        
        serde_json::from_value(result.clone())
            .context(format!("Failed to parse chats. JSON: {}", serde_json::to_string(&result).unwrap_or_else(|_| "?".to_string())))
    }
    
    /// Get messages for a chat
    pub async fn get_messages(&self, chat_id: &str, limit: usize) -> Result<Vec<Message>> {
        let result = self.request("getMessages", json!({
            "chatId": chat_id,
            "limit": limit
        })).await?;
        serde_json::from_value(result).context("Failed to parse messages")
    }
    
    /// Send a message
    pub async fn send_message(&self, chat_id: &str, text: &str) -> Result<()> {
        self.request("sendMessage", json!({
            "chatId": chat_id,
            "text": text
        })).await?;
        Ok(())
    }
    
    /// Download media
    #[allow(dead_code)]
    pub async fn download_media(&self, message_id: &str) -> Result<Vec<u8>> {
        let result = self.request("downloadMedia", json!({
            "messageId": message_id
        })).await?;
        
        // Expecting base64-encoded data
        let base64_str = result.as_str()
            .ok_or_else(|| anyhow!("Media data not a string"))?;
        
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(base64_str)
            .context("Failed to decode base64 media")
    }
}
