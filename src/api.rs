use crate::types::{Message, ModelDescriptor, ModelsResponse, ProviderKind};
use anyhow::{anyhow, Result};
use eventsource_stream::Eventsource;
use futures_util::{stream::BoxStream, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};

const OPENAI_MODELS_ENDPOINT: &str = "https://api.openai.com/v1/models";
const OPENAI_CHAT_COMPLETIONS_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const OPENROUTER_MODELS_ENDPOINT: &str = "https://openrouter.ai/api/v1/models";
const OPENROUTER_CHAT_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const ANTHROPIC_MODELS_ENDPOINT: &str = "https://api.anthropic.com/v1/models";
const ANTHROPIC_MESSAGES_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_MAX_TOKENS: i32 = 1024;

type StreamItem = anyhow::Result<String>;

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

    pub async fn load_models(
        &self,
        provider: ProviderKind,
        api_key: &str,
    ) -> Result<Vec<ModelDescriptor>> {
        match provider {
            ProviderKind::OpenRouter => self.load_openrouter_models(api_key).await,
            ProviderKind::OpenAI => self.load_openai_models(api_key).await,
            ProviderKind::Anthropic => self.load_anthropic_models(api_key).await,
        }
    }

    async fn load_openrouter_models(&self, api_key: &str) -> Result<Vec<ModelDescriptor>> {
        let response = self
            .client
            .get(OPENROUTER_MODELS_ENDPOINT)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/your-username/overlook")
            .header("X-Title", "Overlook Chat")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("OpenRouter API error {}: {}", status, error_text));
        }

        let models_response: ModelsResponse = response.json().await?;
        let mut models: Vec<ModelDescriptor> = models_response
            .data
            .into_iter()
            .map(ModelDescriptor::from)
            .collect();
        models.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(models)
    }

    async fn load_openai_models(&self, api_key: &str) -> Result<Vec<ModelDescriptor>> {
        let response = self
            .client
            .get(OPENAI_MODELS_ENDPOINT)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("OpenAI API error {}: {}", status, error_text));
        }

        #[derive(Deserialize)]
        struct OpenAIModelList {
            data: Vec<OpenAIModel>,
        }

        #[derive(Deserialize)]
        struct OpenAIModel {
            id: String,
            owned_by: Option<String>,
        }

        let mut models: Vec<ModelDescriptor> = response
            .json::<OpenAIModelList>()
            .await?
            .data
            .into_iter()
            .filter(|model| is_supported_openai_model(&model.id))
            .map(|model| ModelDescriptor {
                name: format_model_name(&model.id),
                id: model.id,
                description: model.owned_by.map(|owner| format!("Owned by {}", owner)),
                provider: ProviderKind::OpenAI,
            })
            .collect();

        if models.is_empty() {
            // Provide a sensible default list when the API returns nothing or access is restricted.
            models = fallback_openai_models();
        }

        models.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(models)
    }

    async fn load_anthropic_models(&self, api_key: &str) -> Result<Vec<ModelDescriptor>> {
        let response = self
            .client
            .get(ANTHROPIC_MODELS_ENDPOINT)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Anthropic API error {}: {}", status, error_text));
        }

        #[derive(Deserialize)]
        struct AnthropicModelList {
            data: Vec<AnthropicModel>,
        }

        #[derive(Deserialize)]
        struct AnthropicModel {
            id: String,
        }

        let mut models: Vec<ModelDescriptor> = response
            .json::<AnthropicModelList>()
            .await?
            .data
            .into_iter()
            .filter(|model| model.id.starts_with("claude"))
            .map(|model| ModelDescriptor {
                name: format_model_name(&model.id),
                id: model.id,
                description: None,
                provider: ProviderKind::Anthropic,
            })
            .collect();

        if models.is_empty() {
            models = fallback_anthropic_models();
        }

        models.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(models)
    }

    pub async fn send_chat_message(
        &self,
        provider: ProviderKind,
        api_key: &str,
        model: &str,
        messages: Vec<Message>,
    ) -> Result<String> {
        match provider {
            ProviderKind::OpenRouter => self.send_openrouter_chat(api_key, model, &messages).await,
            ProviderKind::OpenAI => self.send_openai_chat(api_key, model, &messages).await,
            ProviderKind::Anthropic => self.send_anthropic_chat(api_key, model, &messages).await,
        }
    }

    async fn send_openrouter_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let request_body = json!({
            "model": model,
            "messages": openai_style_messages(messages),
            "temperature": 0.7
        });

        let response = self
            .client
            .post(OPENROUTER_CHAT_ENDPOINT)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/your-username/overlook")
            .header("X-Title", "Overlook Chat")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("OpenRouter chat error {}: {}", status, error_text));
        }

        extract_openai_style_response(response.json().await?)
    }

    async fn send_openai_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let request_body = json!({
            "model": model,
            "messages": openai_style_messages(messages),
            "temperature": 0.7
        });

        let response = self
            .client
            .post(OPENAI_CHAT_COMPLETIONS_ENDPOINT)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("OpenAI chat error {}: {}", status, error_text));
        }

        extract_openai_style_response(response.json().await?)
    }

    async fn send_anthropic_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let request_body = anthropic_request_body(messages, model, false);

        let response = self
            .client
            .post(ANTHROPIC_MESSAGES_ENDPOINT)
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Anthropic chat error {}: {}", status, error_text));
        }

        let data: Value = response.json().await?;
        Ok(extract_anthropic_response(&data))
    }

    pub async fn stream_chat_completion(
        &self,
        provider: ProviderKind,
        api_key: &str,
        model: &str,
        messages: Vec<Message>,
    ) -> Result<BoxStream<'static, StreamItem>> {
        match provider {
            ProviderKind::OpenRouter => {
                let request_body = json!({
                    "model": model,
                    "messages": openai_style_messages(&messages),
                    "temperature": 0.7,
                    "stream": true
                });

                let response = self
                    .client
                    .post(OPENROUTER_CHAT_ENDPOINT)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .header("HTTP-Referer", "https://github.com/your-username/overlook")
                    .header("X-Title", "Overlook Chat")
                    .json(&request_body)
                    .send()
                    .await?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(anyhow!(
                        "OpenRouter streaming error {}: {}",
                        status,
                        error_text
                    ));
                }

                let stream = response
                    .bytes_stream()
                    .eventsource()
                    .filter_map(|event| async move {
                        match event {
                            Ok(event) => {
                                if event.data == "[DONE]" {
                                    return None;
                                }

                                match serde_json::from_str::<Value>(&event.data) {
                                    Ok(parsed) => extract_openai_stream_chunk(&parsed),
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to parse OpenRouter SSE JSON: {} for data: {}",
                                            e, event.data
                                        );
                                        None
                                    }
                                }
                            }
                            Err(e) => Some(Err(anyhow!("OpenRouter stream error: {}", e))),
                        }
                    })
                    .boxed();

                Ok(stream)
            }
            ProviderKind::OpenAI => {
                let request_body = json!({
                    "model": model,
                    "messages": openai_style_messages(&messages),
                    "temperature": 0.7,
                    "stream": true
                });

                let response = self
                    .client
                    .post(OPENAI_CHAT_COMPLETIONS_ENDPOINT)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .header("Accept", "text/event-stream")
                    .json(&request_body)
                    .send()
                    .await?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(anyhow!("OpenAI streaming error {}: {}", status, error_text));
                }

                let stream = response
                    .bytes_stream()
                    .eventsource()
                    .filter_map(|event| async move {
                        match event {
                            Ok(event) => {
                                if event.data == "[DONE]" {
                                    return None;
                                }

                                match serde_json::from_str::<Value>(&event.data) {
                                    Ok(parsed) => extract_openai_stream_chunk(&parsed),
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to parse OpenAI SSE JSON: {} for data: {}",
                                            e, event.data
                                        );
                                        None
                                    }
                                }
                            }
                            Err(e) => Some(Err(anyhow!("OpenAI stream error: {}", e))),
                        }
                    })
                    .boxed();

                Ok(stream)
            }
            ProviderKind::Anthropic => {
                let request_body = anthropic_request_body(messages.as_slice(), model, true);

                let response = self
                    .client
                    .post(ANTHROPIC_MESSAGES_ENDPOINT)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", ANTHROPIC_VERSION)
                    .header("Content-Type", "application/json")
                    .header("Accept", "text/event-stream")
                    .json(&request_body)
                    .send()
                    .await?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(anyhow!(
                        "Anthropic streaming error {}: {}",
                        status,
                        error_text
                    ));
                }

                let stream = response
                    .bytes_stream()
                    .eventsource()
                    .filter_map(|event| async move {
                        match event {
                            Ok(event) => match serde_json::from_str::<Value>(&event.data) {
                                Ok(parsed) => extract_anthropic_stream_chunk(&parsed),
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse Anthropic SSE JSON: {} for data: {}",
                                        e, event.data
                                    );
                                    None
                                }
                            },
                            Err(e) => Some(Err(anyhow!("Anthropic stream error: {}", e))),
                        }
                    })
                    .boxed();

                Ok(stream)
            }
        }
    }
}

fn openai_style_messages(messages: &[Message]) -> Vec<Value> {
    messages
        .iter()
        .map(|message| {
            json!({
                "role": message.role,
                "content": message.content,
            })
        })
        .collect()
}

fn anthropic_request_body(messages: &[Message], model: &str, stream: bool) -> Value {
    let mut system_prompt: Option<String> = None;
    let mut formatted_messages = Vec::new();

    for message in messages {
        let role = message.role.to_lowercase();
        match role.as_str() {
            "system" => {
                system_prompt = Some(message.content.clone());
            }
            "assistant" | "user" => {
                let mapped_role = if role == "assistant" {
                    "assistant"
                } else {
                    "user"
                };
                formatted_messages.push(json!({
                    "role": mapped_role,
                    "content": [{"type": "text", "text": message.content}],
                }));
            }
            _ => {
                // Treat any other role as user content to avoid losing context.
                formatted_messages.push(json!({
                    "role": "user",
                    "content": [{"type": "text", "text": message.content}],
                }));
            }
        }
    }

    let mut body = json!({
        "model": model,
        "messages": formatted_messages,
        "max_tokens": DEFAULT_MAX_TOKENS,
        "stream": stream,
    });

    if let Some(system) = system_prompt {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("system".to_string(), Value::String(system));
        }
    }

    body
}

fn extract_openai_style_response(value: Value) -> Result<String> {
    if let Some(choice) = value["choices"].as_array().and_then(|arr| arr.first()) {
        if let Some(content) = choice["message"]["content"].as_str() {
            return Ok(content.to_string());
        }
    }
    Err(anyhow!("Invalid response format"))
}

fn extract_openai_stream_chunk(value: &Value) -> Option<StreamItem> {
    if let Some(choice) = value["choices"].as_array().and_then(|arr| arr.first()) {
        if let Some(delta) = choice["delta"].as_object() {
            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                if !content.is_empty() {
                    return Some(Ok(content.to_string()));
                }
            }
        }
    }
    None
}

fn extract_anthropic_response(value: &Value) -> String {
    if let Some(content) = value["content"].as_array() {
        let mut combined = String::new();
        for block in content {
            if block["type"].as_str() == Some("text") {
                if let Some(text) = block["text"].as_str() {
                    combined.push_str(text);
                }
            }
        }
        return combined;
    }
    String::new()
}

fn extract_anthropic_stream_chunk(value: &Value) -> Option<StreamItem> {
    match value["type"].as_str() {
        Some("content_block_delta") => {
            if let Some(text) = value["delta"]["text"].as_str() {
                if !text.is_empty() {
                    return Some(Ok(text.to_string()));
                }
            }
            None
        }
        Some("error") => {
            let message = value["message"].as_str().unwrap_or("Unknown error");
            Some(Err(anyhow!("Anthropic stream error: {}", message)))
        }
        _ => None,
    }
}

fn format_model_name(id: &str) -> String {
    id.replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_supported_openai_model(id: &str) -> bool {
    const PREFIXES: &[&str] = &["gpt-", "o1-", "o3-"];
    PREFIXES.iter().any(|prefix| id.starts_with(prefix))
}

fn fallback_openai_models() -> Vec<ModelDescriptor> {
    [
        "gpt-4o",
        "gpt-4o-mini",
        "gpt-4.1",
        "gpt-4.1-mini",
        "gpt-3.5-turbo",
    ]
    .iter()
    .map(|id| ModelDescriptor {
        id: id.to_string(),
        name: format_model_name(id),
        description: Some("Default OpenAI model".to_string()),
        provider: ProviderKind::OpenAI,
    })
    .collect()
}

fn fallback_anthropic_models() -> Vec<ModelDescriptor> {
    [
        "claude-3-opus-20240229",
        "claude-3-sonnet-20240229",
        "claude-3-haiku-20240307",
    ]
    .iter()
    .map(|id| ModelDescriptor {
        id: id.to_string(),
        name: format_model_name(id),
        description: Some("Default Anthropic model".to_string()),
        provider: ProviderKind::Anthropic,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_model_name() {
        assert_eq!(format_model_name("gpt-4o"), "Gpt 4o");
        assert_eq!(
            format_model_name("claude-3-sonnet-20240229"),
            "Claude 3 Sonnet 20240229"
        );
    }
}
