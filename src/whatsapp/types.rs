use serde::{Deserialize, Serialize};

/// Events received from WhatsApp service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum WhatsAppEvent {
    #[serde(rename = "qr")]
    QRCode(String),
    
    #[serde(rename = "ready")]
    Ready,
    
    #[serde(rename = "authenticated")]
    Authenticated,
    
    #[serde(rename = "message")]
    MessageReceived(Message),
    
    #[serde(rename = "chat_updated")]
    ChatUpdated(Chat),
    
    #[serde(rename = "disconnected")]
    Disconnected,
}

/// WhatsApp chat representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    
    #[serde(default)]
    pub unread_count: u32,
    
    pub timestamp: i64,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<String>,
}

/// WhatsApp message representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub body: String,
    pub timestamp: i64,
    pub from_me: bool,
    
    #[serde(default)]
    pub has_media: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
}

/// Request to WhatsApp service
#[derive(Debug, Serialize)]
pub struct WhatsAppRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// Response from WhatsApp service
#[derive(Debug, Deserialize)]
pub struct WhatsAppResponse {
    pub id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
