use crate::types::{ModelsResponse, OpenRouterModel};
use anyhow::Result;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::Value;

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

    /// Stream chat completion tokens using proper SSE handling
    /// Returns a stream of tokens that can be consumed in real-time
    pub async fn stream_chat_completion(
        &self,
        api_key: &str,
        model: &str,
        messages: Vec<serde_json::Value>,
    ) -> Result<impl futures_util::Stream<Item = Result<String>>> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
            "stream": true
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

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            if status == 429 {
                return Err(anyhow::anyhow!("Rate limit exceeded. Please wait before trying again. Details: {}", error_text));
            }

            return Err(anyhow::anyhow!("API Error {}: {}", status, error_text));
        }

        // Use proper SSE stream parsing
        let stream = response
            .bytes_stream()
            .eventsource()
            .filter_map(|event| async move {
                match event {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            return None; // End of stream
                        }

                        // Parse the SSE data as JSON
                        match serde_json::from_str::<Value>(&event.data) {
                            Ok(parsed) => {
                                // Extract content from delta
                                if let Some(choices) = parsed["choices"].as_array() {
                                    if let Some(choice) = choices.first() {
                                        if let Some(delta) = choice["delta"].as_object() {
                                            if let Some(content) = delta.get("content") {
                                                if let Some(content_str) = content.as_str() {
                                                    if !content_str.is_empty() {
                                                        return Some(Ok(content_str.to_string()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                None // No content in this chunk
                            }
                            Err(e) => {
                                eprintln!("Failed to parse SSE JSON: {} for data: {}", e, event.data);
                                None // Skip malformed chunks
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("SSE stream error: {}", e);
                        Some(Err(anyhow::anyhow!("Stream error: {}", e)))
                    }
                }
            });

        Ok(stream)
    }
}