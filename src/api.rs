use crate::types::{ModelsResponse, OpenRouterModel};
use anyhow::Result;

pub struct ApiClient {
    client: reqwest::Client,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn load_models(&self, api_key: &str) -> Result<Vec<OpenRouterModel>> {
        let response = self.client
            .get("https://openrouter.ai/api/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/your-username/overlook")
            .header("X-Title", "Overlook Chat")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("API Error {}: {}", status, error_text));
        }

        let models_response: ModelsResponse = response.json().await?;
        Ok(models_response.data)
    }

    pub async fn send_chat_message(
        &self,
        api_key: &str,
        model: &str,
        messages: Vec<serde_json::Value>,
    ) -> Result<String> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7
        });

        let response = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/your-username/overlook")
            .header("X-Title", "Overlook Chat")
            .json(&request_body)
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;

        if let Some(choice) = data["choices"].as_array().and_then(|arr| arr.first()) {
            if let Some(content) = choice["message"]["content"].as_str() {
                return Ok(content.to_string());
            }
        }

        Err(anyhow::anyhow!("Invalid response format"))
    }
}