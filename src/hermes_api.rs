use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::runtime::Runtime;

// =============================================================================
// Hermes HTTP Bridge API Client
// =============================================================================

#[derive(Serialize)]
struct ChatRequest<'a> {
    message: &'a str,
    session_id: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
}

#[derive(Clone)]
pub struct HermesClient {
    base_url: String,
}

impl HermesClient {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:8765".to_string(),
        }
    }

    /// Create a client with a custom base URL (e.g. "http://192.168.1.50:8765")
    /// The URL should NOT include the /chat path — this client appends it.
    /// Trailing slashes are stripped automatically.
    pub fn with_base_url(mut base_url: String) -> Self {
        while base_url.ends_with('/') {
            base_url.pop();
        }
        Self { base_url }
    }

    /// Send a message to the Hermes agent and return the response with session ID.
    /// Uses the sync wrapper which blocks on a tokio runtime.
    pub fn send_message(
        &self,
        message: &str,
        session_id: &str,
    ) -> Result<(String, String)> {
        let rt = get_tokio_runtime();
        rt.block_on(self.send_message_async(message, session_id))
    }

    /// Async version of send_message
    pub async fn send_message_async(
        &self,
        message: &str,
        session_id: &str,
    ) -> Result<(String, String)> {
        let url = format!("{}/chat", self.base_url);
        
        let request_body = ChatRequest {
            message,
            session_id,
        };

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to Hermes agent at {}: {}", url, e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Hermes agent returned error {}: {}", status, error_text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Hermes response: {}", e))?;

        Ok((chat_response.response, chat_response.session_id))
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