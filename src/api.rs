use crate::types::{Message, ModelDescriptor, ModelsResponse, ProviderKind};
use anyhow::{anyhow, Result};
use eventsource_stream::Eventsource;
use futures_util::{stream::BoxStream, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

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

    /// Determines if an error should be retried based on the status code
    fn is_retryable_error(error: &anyhow::Error) -> bool {
        if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
            // Retry on network/connection errors
            if reqwest_error.is_timeout() || reqwest_error.is_connect() {
                return true;
            }

            // Check HTTP status codes
            if let Some(status) = reqwest_error.status() {
                let code = status.as_u16();
                // Retry on rate limiting and server errors
                return code == 429 || code == 500 || code == 502 || code == 503 || code == 504;
            }
        }
        false
    }

    /// Creates a retry strategy with exponential backoff and jitter
    fn retry_strategy() -> impl Iterator<Item = Duration> {
        ExponentialBackoff::from_millis(1000) // Start at 1 second
            .max_delay(Duration::from_secs(8)) // Cap at 8 seconds
            .take(3) // Max 3 attempts (initial + 2 retries)
            .map(jitter) // Add jitter to each duration
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
            ProviderKind::LMStudio => self.load_lmstudio_models(api_key).await,
            ProviderKind::Ollama => self.load_ollama_models(api_key).await,
        }
    }

    async fn load_openrouter_models(&self, api_key: &str) -> Result<Vec<ModelDescriptor>> {
        let api_key = api_key.to_string();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let api_key = api_key.clone();
            let client = client.clone();
            async move {
                let response = client
                    .get(OPENROUTER_MODELS_ENDPOINT)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("HTTP-Referer", "https://github.com/your-username/overlook")
                    .header("X-Title", "Overlook Chat")
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("OpenRouter API error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying OpenRouter models request: {}", error);
                    }
                    return Err(error);
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
        })
        .await
    }

    async fn load_openai_models(&self, api_key: &str) -> Result<Vec<ModelDescriptor>> {
        #[derive(Deserialize)]
        struct OpenAIModelList {
            data: Vec<OpenAIModel>,
        }

        #[derive(Deserialize)]
        struct OpenAIModel {
            id: String,
            owned_by: Option<String>,
        }

        let api_key = api_key.to_string();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let api_key = api_key.clone();
            let client = client.clone();
            async move {
                let response = client
                    .get(OPENAI_MODELS_ENDPOINT)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("OpenAI API error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying OpenAI models request: {}", error);
                    }
                    return Err(error);
                }

                let mut models: Vec<ModelDescriptor> = response
                    .json::<OpenAIModelList>()
                    .await?
                    .data
                    .into_iter()
                    .filter(|model| is_supported_openai_model(&model.id))
                    .map(|model| {
                        let context_length = openai_context_length(&model.id);
                        ModelDescriptor {
                            name: format_model_name(&model.id),
                            id: model.id.clone(),
                            description: model.owned_by.map(|owner| format!("Owned by {}", owner)),
                            provider: ProviderKind::OpenAI,
                            context_length,
                            max_output_tokens: None,
                            tokenizer: Some(openai_tokenizer_name(&model.id)),
                        }
                    })
                    .collect();

                if models.is_empty() {
                    // Provide a sensible default list when the API returns nothing or access is restricted.
                    models = fallback_openai_models();
                }

                models.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(models)
            }
        })
        .await
    }

    async fn load_anthropic_models(&self, api_key: &str) -> Result<Vec<ModelDescriptor>> {
        #[derive(Deserialize)]
        struct AnthropicModelList {
            data: Vec<AnthropicModel>,
        }

        #[derive(Deserialize)]
        struct AnthropicModel {
            id: String,
        }

        let api_key = api_key.to_string();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let api_key = api_key.clone();
            let client = client.clone();
            async move {
                let response = client
                    .get(ANTHROPIC_MODELS_ENDPOINT)
                    .header("x-api-key", &api_key)
                    .header("anthropic-version", ANTHROPIC_VERSION)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("Anthropic API error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying Anthropic models request: {}", error);
                    }
                    return Err(error);
                }

                let mut models: Vec<ModelDescriptor> = response
                    .json::<AnthropicModelList>()
                    .await?
                    .data
                    .into_iter()
                    .filter(|model| model.id.starts_with("claude"))
                    .map(|model| ModelDescriptor {
                        name: format_model_name(&model.id),
                        id: model.id.clone(),
                        description: None,
                        provider: ProviderKind::Anthropic,
                        context_length: anthropic_context_length(&model.id),
                        max_output_tokens: None,
                        tokenizer: Some(anthropic_tokenizer_name(&model.id)),
                    })
                    .collect();

                if models.is_empty() {
                    models = fallback_anthropic_models();
                }

                models.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(models)
            }
        })
        .await
    }

    async fn load_lmstudio_models(&self, base_url: &str) -> Result<Vec<ModelDescriptor>> {
        #[derive(Deserialize)]
        struct LMStudioModelList {
            data: Vec<LMStudioModel>,
        }

        #[derive(Deserialize)]
        struct LMStudioModel {
            id: String,
        }

        let base_url = base_url.trim_end_matches('/').to_string();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let base_url = base_url.clone();
            let client = client.clone();
            async move {
                let endpoint = format!("{}/v1/models", base_url);
                let response = client
                    .get(&endpoint)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("LM Studio API error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying LM Studio models request: {}", error);
                    }
                    return Err(error);
                }

                let mut models: Vec<ModelDescriptor> = response
                    .json::<LMStudioModelList>()
                    .await?
                    .data
                    .into_iter()
                    .map(|model| ModelDescriptor {
                        name: model.id.clone(),
                        id: model.id.clone(),
                        description: Some("LM Studio model".to_string()),
                        provider: ProviderKind::LMStudio,
                        context_length: Some(4096),
                        max_output_tokens: None,
                        tokenizer: None,
                    })
                    .collect();

                models.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(models)
            }
        })
        .await
    }

    async fn load_ollama_models(&self, base_url: &str) -> Result<Vec<ModelDescriptor>> {
        #[derive(Deserialize)]
        struct OllamaModelList {
            models: Vec<OllamaModel>,
        }

        #[derive(Deserialize)]
        struct OllamaModel {
            name: String,
        }

        let base_url = base_url.trim_end_matches('/').to_string();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let base_url = base_url.clone();
            let client = client.clone();
            async move {
                let endpoint = format!("{}/api/tags", base_url);
                let response = client
                    .get(&endpoint)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("Ollama API error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying Ollama models request: {}", error);
                    }
                    return Err(error);
                }

                let mut models: Vec<ModelDescriptor> = response
                    .json::<OllamaModelList>()
                    .await?
                    .models
                    .into_iter()
                    .map(|model| ModelDescriptor {
                        name: model.name.clone(),
                        id: model.name.clone(),
                        description: Some("Ollama model".to_string()),
                        provider: ProviderKind::Ollama,
                        context_length: Some(4096),
                        max_output_tokens: None,
                        tokenizer: None,
                    })
                    .collect();

                models.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(models)
            }
        })
        .await
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
            ProviderKind::LMStudio => self.send_lmstudio_chat(api_key, model, &messages).await,
            ProviderKind::Ollama => self.send_ollama_chat(api_key, model, &messages).await,
        }
    }

    async fn send_openrouter_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let api_key = api_key.to_string();
        let model = model.to_string();
        let messages = messages.to_vec();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let api_key = api_key.clone();
            let model = model.clone();
            let messages = messages.clone();
            let client = client.clone();
            async move {
                let request_body = json!({
                    "model": model,
                    "messages": openai_style_messages(&messages),
                    "temperature": 0.7
                });

                let response = client
                    .post(OPENROUTER_CHAT_ENDPOINT)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .header("HTTP-Referer", "https://github.com/your-username/overlook")
                    .header("X-Title", "Overlook Chat")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("OpenRouter chat error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying OpenRouter chat request: {}", error);
                    }
                    return Err(error);
                }

                extract_openai_style_response(response.json().await?)
            }
        })
        .await
    }

    async fn send_openai_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let api_key = api_key.to_string();
        let model = model.to_string();
        let messages = messages.to_vec();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let api_key = api_key.clone();
            let model = model.clone();
            let messages = messages.clone();
            let client = client.clone();
            async move {
                let request_body = json!({
                    "model": model,
                    "messages": openai_style_messages(&messages),
                    "temperature": 0.7
                });

                let response = client
                    .post(OPENAI_CHAT_COMPLETIONS_ENDPOINT)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("OpenAI chat error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying OpenAI chat request: {}", error);
                    }
                    return Err(error);
                }

                extract_openai_style_response(response.json().await?)
            }
        })
        .await
    }

    async fn send_anthropic_chat(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let api_key = api_key.to_string();
        let model = model.to_string();
        let messages = messages.to_vec();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let api_key = api_key.clone();
            let model = model.clone();
            let messages = messages.clone();
            let client = client.clone();
            async move {
                let request_body = anthropic_request_body(&messages, &model, false);

                let response = client
                    .post(ANTHROPIC_MESSAGES_ENDPOINT)
                    .header("x-api-key", &api_key)
                    .header("anthropic-version", ANTHROPIC_VERSION)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("Anthropic chat error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying Anthropic chat request: {}", error);
                    }
                    return Err(error);
                }

                let data: Value = response.json().await?;
                Ok(extract_anthropic_response(&data))
            }
        })
        .await
    }

    async fn send_lmstudio_chat(
        &self,
        base_url: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let base_url = base_url.trim_end_matches('/').to_string();
        let model = model.to_string();
        let messages = messages.to_vec();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let base_url = base_url.clone();
            let model = model.clone();
            let messages = messages.clone();
            let client = client.clone();
            async move {
                let endpoint = format!("{}/v1/chat/completions", base_url);
                let request_body = json!({
                    "model": model,
                    "messages": openai_style_messages(&messages),
                    "temperature": 0.7
                });

                let response = client
                    .post(&endpoint)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("LM Studio chat error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying LM Studio chat request: {}", error);
                    }
                    return Err(error);
                }

                extract_openai_style_response(response.json().await?)
            }
        })
        .await
    }

    async fn send_ollama_chat(
        &self,
        base_url: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<String> {
        let base_url = base_url.trim_end_matches('/').to_string();
        let model = model.to_string();
        let messages = messages.to_vec();
        let client = self.client.clone();

        Retry::spawn(Self::retry_strategy(), move || {
            let base_url = base_url.clone();
            let model = model.clone();
            let messages = messages.clone();
            let client = client.clone();
            async move {
                let endpoint = format!("{}/api/chat", base_url);

                // Convert messages to Ollama format
                let ollama_messages: Vec<Value> = messages.iter().map(|m| {
                    json!({
                        "role": m.role,
                        "content": m.content
                    })
                }).collect();

                let request_body = json!({
                    "model": model,
                    "messages": ollama_messages,
                    "stream": false
                });

                let response = client
                    .post(&endpoint)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| anyhow!(e))?;

                let status = response.status();
                if !status.is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    let error = anyhow!("Ollama chat error {}: {}", status, error_text);
                    if Self::is_retryable_error(&error) {
                        eprintln!("Retrying Ollama chat request: {}", error);
                    }
                    return Err(error);
                }

                let data: Value = response.json().await?;
                // Ollama returns the response in data["message"]["content"]
                if let Some(content) = data["message"]["content"].as_str() {
                    Ok(content.to_string())
                } else {
                    Err(anyhow!("Invalid Ollama response format"))
                }
            }
        })
        .await
    }

    pub async fn stream_chat_completion(
        &self,
        provider: ProviderKind,
        api_key: &str,
        model: &str,
        messages: Vec<Message>,
    ) -> Result<BoxStream<'static, StreamItem>> {
        match provider {
            ProviderKind::LMStudio => {
                let base_url = api_key.trim_end_matches('/').to_string();
                let model = model.to_string();
                let client = self.client.clone();

                // Retry the initial connection with backoff
                let response = Retry::spawn(Self::retry_strategy(), move || {
                    let base_url = base_url.clone();
                    let model = model.clone();
                    let messages = messages.clone();
                    let client = client.clone();
                    async move {
                        let endpoint = format!("{}/v1/chat/completions", base_url);
                        let request_body = json!({
                            "model": model,
                            "messages": openai_style_messages(&messages),
                            "temperature": 0.7,
                            "stream": true
                        });

                        let response = client
                            .post(&endpoint)
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!(e))?;

                        let status = response.status();
                        if !status.is_success() {
                            let error_text = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            let error = anyhow!(
                                "LM Studio streaming error {}: {}",
                                status,
                                error_text
                            );
                            if Self::is_retryable_error(&error) {
                                eprintln!("Retrying LM Studio streaming connection: {}", error);
                            }
                            return Err(error);
                        }

                        Ok(response)
                    }
                })
                .await?;

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
                                            "Failed to parse LM Studio SSE JSON: {} for data: {}",
                                            e, event.data
                                        );
                                        None
                                    }
                                }
                            }
                            Err(e) => Some(Err(anyhow!("LM Studio stream error: {}", e))),
                        }
                    })
                    .boxed();

                Ok(stream)
            }
            ProviderKind::Ollama => {
                let base_url = api_key.trim_end_matches('/').to_string();
                let model = model.to_string();
                let client = self.client.clone();

                // Retry the initial connection with backoff
                let response = Retry::spawn(Self::retry_strategy(), move || {
                    let base_url = base_url.clone();
                    let model = model.clone();
                    let messages = messages.clone();
                    let client = client.clone();
                    async move {
                        let endpoint = format!("{}/api/chat", base_url);

                        // Convert messages to Ollama format
                        let ollama_messages: Vec<Value> = messages.iter().map(|m| {
                            json!({
                                "role": m.role,
                                "content": m.content
                            })
                        }).collect();

                        let request_body = json!({
                            "model": model,
                            "messages": ollama_messages,
                            "stream": true
                        });

                        let response = client
                            .post(&endpoint)
                            .header("Content-Type", "application/json")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!(e))?;

                        let status = response.status();
                        if !status.is_success() {
                            let error_text = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            let error = anyhow!(
                                "Ollama streaming error {}: {}",
                                status,
                                error_text
                            );
                            if Self::is_retryable_error(&error) {
                                eprintln!("Retrying Ollama streaming connection: {}", error);
                            }
                            return Err(error);
                        }

                        Ok(response)
                    }
                })
                .await?;

                let stream = response
                    .bytes_stream()
                    .filter_map(|result| async move {
                        match result {
                            Ok(bytes) => {
                                // Ollama streams newline-delimited JSON
                                let text = String::from_utf8_lossy(&bytes);
                                for line in text.lines() {
                                    if line.trim().is_empty() {
                                        continue;
                                    }
                                    match serde_json::from_str::<Value>(line) {
                                        Ok(parsed) => {
                                            // Check if stream is done
                                            if parsed["done"].as_bool() == Some(true) {
                                                return None;
                                            }
                                            // Extract content from message
                                            if let Some(content) = parsed["message"]["content"].as_str() {
                                                if !content.is_empty() {
                                                    return Some(Ok(content.to_string()));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Failed to parse Ollama stream JSON: {} for line: {}",
                                                e, line
                                            );
                                        }
                                    }
                                }
                                None
                            }
                            Err(e) => Some(Err(anyhow!("Ollama stream error: {}", e))),
                        }
                    })
                    .boxed();

                Ok(stream)
            }
            ProviderKind::OpenRouter => {
                let api_key = api_key.to_string();
                let model = model.to_string();
                let client = self.client.clone();

                // Retry the initial connection with backoff
                let response = Retry::spawn(Self::retry_strategy(), move || {
                    let api_key = api_key.clone();
                    let model = model.clone();
                    let messages = messages.clone();
                    let client = client.clone();
                    async move {
                        let request_body = json!({
                            "model": model,
                            "messages": openai_style_messages(&messages),
                            "temperature": 0.7,
                            "stream": true
                        });

                        let response = client
                            .post(OPENROUTER_CHAT_ENDPOINT)
                            .header("Authorization", format!("Bearer {}", api_key))
                            .header("Content-Type", "application/json")
                            .header("HTTP-Referer", "https://github.com/your-username/overlook")
                            .header("X-Title", "Overlook Chat")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!(e))?;

                        let status = response.status();
                        if !status.is_success() {
                            let error_text = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            let error = anyhow!(
                                "OpenRouter streaming error {}: {}",
                                status,
                                error_text
                            );
                            if Self::is_retryable_error(&error) {
                                eprintln!("Retrying OpenRouter streaming connection: {}", error);
                            }
                            return Err(error);
                        }

                        Ok(response)
                    }
                })
                .await?;

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
                let api_key = api_key.to_string();
                let model = model.to_string();
                let client = self.client.clone();

                // Retry the initial connection with backoff
                let response = Retry::spawn(Self::retry_strategy(), move || {
                    let api_key = api_key.clone();
                    let model = model.clone();
                    let messages = messages.clone();
                    let client = client.clone();
                    async move {
                        let request_body = json!({
                            "model": model,
                            "messages": openai_style_messages(&messages),
                            "temperature": 0.7,
                            "stream": true
                        });

                        let response = client
                            .post(OPENAI_CHAT_COMPLETIONS_ENDPOINT)
                            .header("Authorization", format!("Bearer {}", api_key))
                            .header("Content-Type", "application/json")
                            .header("Accept", "text/event-stream")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!(e))?;

                        let status = response.status();
                        if !status.is_success() {
                            let error_text = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            let error = anyhow!("OpenAI streaming error {}: {}", status, error_text);
                            if Self::is_retryable_error(&error) {
                                eprintln!("Retrying OpenAI streaming connection: {}", error);
                            }
                            return Err(error);
                        }

                        Ok(response)
                    }
                })
                .await?;

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
                let api_key = api_key.to_string();
                let model = model.to_string();
                let client = self.client.clone();

                // Retry the initial connection with backoff
                let response = Retry::spawn(Self::retry_strategy(), move || {
                    let api_key = api_key.clone();
                    let model = model.clone();
                    let messages = messages.clone();
                    let client = client.clone();
                    async move {
                        let request_body = anthropic_request_body(messages.as_slice(), &model, true);

                        let response = client
                            .post(ANTHROPIC_MESSAGES_ENDPOINT)
                            .header("x-api-key", &api_key)
                            .header("anthropic-version", ANTHROPIC_VERSION)
                            .header("Content-Type", "application/json")
                            .header("Accept", "text/event-stream")
                            .json(&request_body)
                            .send()
                            .await
                            .map_err(|e| anyhow!(e))?;

                        let status = response.status();
                        if !status.is_success() {
                            let error_text = response
                                .text()
                                .await
                                .unwrap_or_else(|_| "Unknown error".to_string());
                            let error = anyhow!(
                                "Anthropic streaming error {}: {}",
                                status,
                                error_text
                            );
                            if Self::is_retryable_error(&error) {
                                eprintln!("Retrying Anthropic streaming connection: {}", error);
                            }
                            return Err(error);
                        }

                        Ok(response)
                    }
                })
                .await?;

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
        context_length: openai_context_length(id),
        max_output_tokens: None,
        tokenizer: Some(openai_tokenizer_name(id)),
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
        context_length: anthropic_context_length(id),
        max_output_tokens: None,
        tokenizer: Some(anthropic_tokenizer_name(id)),
    })
    .collect()
}

fn openai_context_length(model_id: &str) -> Option<usize> {
    const DEFAULT_CONTEXT: usize = 128_000;
    const MAPPINGS: &[(&str, usize)] = &[
        ("gpt-3.5-turbo", 16_385),
        ("gpt-4o-mini", 128_000),
        ("gpt-4o", 128_000),
        ("gpt-4.1-mini", 128_000),
        ("gpt-4.1", 128_000),
        ("gpt-4-turbo", 128_000),
        ("o1-", 128_000),
        ("o3-", 128_000),
    ];

    for (prefix, value) in MAPPINGS {
        if model_id.starts_with(prefix) {
            return Some(*value);
        }
    }

    Some(DEFAULT_CONTEXT)
}

fn openai_tokenizer_name(model_id: &str) -> String {
    model_id.to_string()
}

fn anthropic_context_length(_model_id: &str) -> Option<usize> {
    // Claude 3 models support up to 200k context tokens.
    Some(200_000)
}

fn anthropic_tokenizer_name(_model_id: &str) -> String {
    "claude-3".to_string()
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
