use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::runtime::Runtime;

// =============================================================================
// Hermes API Server Client (OpenAI-compatible /v1/chat/completions)
// =============================================================================

/// OpenAI-compatible chat completion request body
#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible chat completion response
#[derive(Deserialize, Debug)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize, Debug)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Clone)]
pub struct HermesClient {
    base_url: String,
    api_key: String,
}

impl HermesClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:8642".to_string(),
            api_key: String::new(),
        }
    }

    /// Create a client with a custom base URL (e.g. "http://192.168.1.50:8642")
    /// The URL should NOT include /v1 paths — this client appends them.
    /// Trailing slashes are stripped automatically.
    pub fn with_base_url(mut base_url: String) -> Self {
        while base_url.ends_with('/') {
            base_url.pop();
        }
        Self {
            base_url,
            api_key: String::new(),
        }
    }

    /// Create a client with both base URL and API key
    pub fn with_base_url_and_key(base_url: String, api_key: String) -> Self {
        let mut url = base_url;
        while url.ends_with('/') {
            url.pop();
        }
        Self {
            base_url: url,
            api_key,
        }
    }

    /// Send a message to the Hermes agent and return the response text.
    /// Uses the OpenAI-compatible /v1/chat/completions endpoint.
    pub fn send_message(
        &self,
        message: &str,
        session_id: &str,
    ) -> Result<(String, String)> {
        let rt = get_tokio_runtime();
        rt.block_on(self.send_message_async(message, session_id))
    }

    /// Async version — sends to /v1/chat/completions and parses the response.
    /// Returns (response_text, session_id).
    /// The session_id is extracted from the X-Hermes-Session-Id response header
    /// when available, otherwise the provided session_id is returned unchanged.
    pub async fn send_message_async(
        &self,
        message: &str,
        session_id: &str,
    ) -> Result<(String, String)> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let request_body = ChatCompletionRequest {
            model: "hermes-agent".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            stream: false,
        };

        let client = reqwest::Client::new();
        let mut request_builder = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Hermes-Session-Id", session_id);

        if !self.api_key.is_empty() {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let response = request_builder
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to Hermes agent at {}: {}", url, e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Hermes agent returned error {}: {}",
                status,
                error_text
            ));
        }

        // Extract session ID from response header if present
        let returned_session_id = response
            .headers()
            .get("X-Hermes-Session-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| session_id.to_string());

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Hermes response: {}", e))?;

        let content = chat_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("No response content from Hermes agent"))?;

        Ok((content, returned_session_id))
    }

    /// Check if the Hermes agent is reachable
    pub fn is_available(&self) -> bool {
        let rt = get_tokio_runtime();
        rt.block_on(self.check_available_async())
    }

    async fn check_available_async(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/health", self.base_url);

        match client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

// =============================================================================
// Tokio Runtime Management
// =============================================================================

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_tokio_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create Tokio runtime"))
}