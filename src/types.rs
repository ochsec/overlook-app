use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    OpenRouter,
    OpenAI,
    Anthropic,
    LMStudio,
    Ollama,
    Hermes,
}

impl ProviderKind {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::OpenAI => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::LMStudio => "lmstudio",
            ProviderKind::Ollama => "ollama",
            ProviderKind::Hermes => "hermes",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "OpenRouter",
            ProviderKind::OpenAI => "OpenAI",
            ProviderKind::Anthropic => "Anthropic",
            ProviderKind::LMStudio => "LM Studio",
            ProviderKind::Ollama => "Ollama",
            ProviderKind::Hermes => "Hermes Agent",
        }
    }

    pub fn all() -> [ProviderKind; 6] {
        [
            ProviderKind::OpenRouter,
            ProviderKind::OpenAI,
            ProviderKind::Anthropic,
            ProviderKind::LMStudio,
            ProviderKind::Ollama,
            ProviderKind::Hermes,
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
    #[serde(default)]
    pub lmstudio_url: String,
    #[serde(default)]
    pub ollama_url: String,
    #[serde(default = "default_hermes_url")]
    pub hermes_url: String,
    #[serde(default)]
    pub hermes_api_key: String,
}

fn default_hermes_url() -> String {
    "http://localhost:8642".to_string()
}

impl ProviderKeys {
    pub fn get(&self, provider: ProviderKind) -> &str {
        match provider {
            ProviderKind::OpenRouter => &self.openrouter,
            ProviderKind::OpenAI => &self.openai,
            ProviderKind::Anthropic => &self.anthropic,
            ProviderKind::LMStudio => &self.lmstudio_url,
            ProviderKind::Ollama => &self.ollama_url,
            ProviderKind::Hermes => &self.hermes_url,
        }
    }

    pub fn get_mut(&mut self, provider: ProviderKind) -> &mut String {
        match provider {
            ProviderKind::OpenRouter => &mut self.openrouter,
            ProviderKind::OpenAI => &mut self.openai,
            ProviderKind::Anthropic => &mut self.anthropic,
            ProviderKind::LMStudio => &mut self.lmstudio_url,
            ProviderKind::Ollama => &mut self.ollama_url,
            ProviderKind::Hermes => &mut self.hermes_url,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub server_name: String,
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingToolCall {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub server_name: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_results: Option<Vec<SearchResult>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
    /// Auto-generated title for user messages (used in turn headers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Message {
    pub fn new_user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            timestamp: Utc::now(),
            attachments: Vec::new(),
            provider: None,
            model: None,
            search_results: None,
            tool_calls: None,
            tool_result: None,
            title: None,
        }
    }

    pub fn new_assistant(content: String, provider: ProviderKind, model: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            timestamp: Utc::now(),
            attachments: Vec::new(),
            provider: Some(provider),
            model: Some(model),
            search_results: None,
            tool_calls: None,
            tool_result: None,
            title: None,
        }
    }

    pub fn new_assistant_with_tool_calls(
        content: String,
        provider: ProviderKind,
        model: String,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            timestamp: Utc::now(),
            attachments: Vec::new(),
            provider: Some(provider),
            model: Some(model),
            search_results: None,
            tool_calls: Some(tool_calls),
            tool_result: None,
            title: None,
        }
    }

    pub fn new_tool_result(result: ToolResult) -> Self {
        Self {
            role: "tool".to_string(),
            content: result.content.clone(),
            timestamp: Utc::now(),
            attachments: Vec::new(),
            provider: None,
            model: None,
            search_results: None,
            tool_calls: None,
            tool_result: Some(result),
            title: None,
        }
    }
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

/// A conversation thread with the Hermes Agent
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentThread {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type AgentThreadMap = HashMap<String, AgentThread>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub server_name: String,
    pub description: String,
    pub schema: Option<serde_json::Value>,
}
