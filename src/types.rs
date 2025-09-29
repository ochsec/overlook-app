use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attachment {
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
    pub data: AttachmentData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AttachmentData {
    Image { base64: String },
    Text { content: String }, // For processed PDFs and text files
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub attachments: Vec<Attachment>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub model: String,
    pub folder: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

pub type ChatMap = HashMap<String, Chat>;
pub type FolderMap = HashMap<String, String>;
