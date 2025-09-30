use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    OpenRouter,
    OpenAI,
    Anthropic,
}

impl ProviderKind {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::OpenAI => "openai",
            ProviderKind::Anthropic => "anthropic",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "OpenRouter",
            ProviderKind::OpenAI => "OpenAI",
            ProviderKind::Anthropic => "Anthropic",
        }
    }

    pub fn all() -> [ProviderKind; 3] {
        [
            ProviderKind::OpenRouter,
            ProviderKind::OpenAI,
            ProviderKind::Anthropic,
        ]
    }
}

impl Default for ProviderKind {
    fn default() -> Self {
        ProviderKind::OpenRouter
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProviderKeys {
    pub openrouter: String,
    pub openai: String,
    pub anthropic: String,
}

impl ProviderKeys {
    pub fn get(&self, provider: ProviderKind) -> &str {
        match provider {
            ProviderKind::OpenRouter => &self.openrouter,
            ProviderKind::OpenAI => &self.openai,
            ProviderKind::Anthropic => &self.anthropic,
        }
    }

    pub fn get_mut(&mut self, provider: ProviderKind) -> &mut String {
        match provider {
            ProviderKind::OpenRouter => &mut self.openrouter,
            ProviderKind::OpenAI => &mut self.openai,
            ProviderKind::Anthropic => &mut self.anthropic,
        }
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub model: String,
    #[serde(default)]
    pub provider: ProviderKind,
    pub project_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub architecture: Option<OpenRouterArchitecture>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OpenRouterArchitecture {
    pub tokenizer: Option<String>,
    pub context_length: Option<usize>,
    #[serde(alias = "max_output_length")]
    pub max_output_tokens: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

#[derive(Clone, Debug)]
pub struct ModelDescriptor {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: ProviderKind,
    pub context_length: Option<usize>,
    pub max_output_tokens: Option<usize>,
    pub tokenizer: Option<String>,
}

impl From<OpenRouterModel> for ModelDescriptor {
    fn from(value: OpenRouterModel) -> Self {
        let architecture = value.architecture.unwrap_or_default();
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            provider: ProviderKind::OpenRouter,
            context_length: architecture.context_length,
            max_output_tokens: architecture.max_output_tokens,
            tokenizer: architecture.tokenizer,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: now,
            updated_at: now,
            color: None,
        }
    }
}

pub type ChatMap = HashMap<String, Chat>;
pub type ProjectMap = HashMap<String, Project>;
// Legacy type for migration
pub type FolderMap = HashMap<String, String>;
