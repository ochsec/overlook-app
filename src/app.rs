use crate::api::ApiClient;
use crate::attachments::AttachmentProcessor;
use crate::export::Exporter;
use crate::storage::Storage;
use crate::theme::{Theme, ThemeType};
use crate::tokens::{TokenCounter, TokenUsage};
use crate::types::{
    Chat, ChatMap, Message, ModelDescriptor, Project, ProjectMap, ProviderKeys, ProviderKind,
};
use chrono::Utc;
use eframe::egui;
use poll_promise::Promise;
use std::sync::mpsc;

#[derive(Clone, Copy, PartialEq)]
pub enum ProjectModalMode {
    Create,
    Rename,
}

pub struct OverlookApp {
    pub api_keys: ProviderKeys,
    pub current_chat_id: Option<String>,
    pub current_provider: ProviderKind,
    pub current_model: String,
    pub message_input: String,
    pub chats: ChatMap,
    pub projects: ProjectMap,
    pub current_project_id: Option<String>,
    pub available_models: Vec<ModelDescriptor>,
    pub loading_models: bool,
    pub sending_message: bool,
    pub show_project_modal: bool,
    pub new_project_name: String,
    pub project_modal_mode: ProjectModalMode,
    pub project_modal_target_id: Option<String>,
    pub show_organize_modal: bool,
    pub show_export_modal: bool,
    pub show_api_settings_window: bool,
    pub api_key_masked: bool,
    pub rt: Option<tokio::runtime::Runtime>,
    pub api_promise: Option<Promise<Result<String, String>>>,
    pub current_theme_type: ThemeType,
    pub current_theme: Theme,
    pub font_size: f32,
    pub selected_attachments: Vec<String>, // File paths of selected attachments
    pub file_picker_promise: Option<Promise<Option<Vec<String>>>>, // Async file picker
    pub streaming_enabled: bool,
    pub streaming_message: String, // Current streaming message content
    pub streaming_active: bool,
    pub stream_completed: bool, // True when API stream ends, but chars may still be displaying
    pub streaming_promise: Option<Promise<Result<String, String>>>,
    pub streaming_result: Option<Result<String, String>>, // Store result when promise completes
    pub streaming_chat_id: Option<String>, // Track chat that initiated the current stream
    pub streaming_message_base_len: usize, // Message count before the assistant reply is revealed
    pub streaming_message_persisted: bool, // True once the streamed reply is saved to disk
    pub token_receiver: Option<mpsc::Receiver<String>>,
    pub char_buffer: String, // Buffer for smooth character-by-character display
    pub char_display_index: usize, // Current position in character display
    pub last_char_time: std::time::Instant, // For timing character releases
    pub token_counter: TokenCounter,
    pub token_usage: TokenUsage,
    pub last_request_provider: Option<ProviderKind>,
    pub last_request_model: Option<String>,
    pub last_model_descriptor: Option<ModelDescriptor>,
    api_client: ApiClient,
}

impl Default for OverlookApp {
    fn default() -> Self {
        let theme_type = Storage::load_theme().unwrap_or_default();
        let theme = Theme::from_theme_type(&theme_type);
        Self {
            api_keys: ProviderKeys::default(),
            current_chat_id: None,
            current_provider: ProviderKind::default(),
            current_model: String::default(),
            message_input: String::default(),
            chats: ChatMap::default(),
            projects: ProjectMap::default(),
            current_project_id: None,
            available_models: Vec::default(),
            loading_models: bool::default(),
            sending_message: bool::default(),
            show_project_modal: bool::default(),
            new_project_name: String::default(),
            project_modal_mode: ProjectModalMode::Create,
            project_modal_target_id: None,
            show_organize_modal: bool::default(),
            show_export_modal: bool::default(),
            show_api_settings_window: false,
            api_key_masked: true,
            rt: None,
            api_promise: None,
            current_theme_type: theme_type,
            current_theme: theme,
            font_size: 14.0, // Default 14pt font size
            selected_attachments: Vec::new(),
            file_picker_promise: None,
            streaming_enabled: true, // Enable streaming by default
            streaming_message: String::new(),
            streaming_active: false,
            stream_completed: false,
            streaming_promise: None,
            streaming_result: None,
            streaming_chat_id: None,
            streaming_message_base_len: 0,
            streaming_message_persisted: false,
            token_receiver: None,
            char_buffer: String::new(),
            char_display_index: 0,
            last_char_time: std::time::Instant::now(),
            token_counter: TokenCounter::default(),
            token_usage: TokenUsage::default(),
            last_request_provider: None,
            last_request_model: None,
            last_model_descriptor: None,
            api_client: ApiClient::new(),
        }
    }
}

impl OverlookApp {
    /// Create an optimized Tokio runtime based on the target platform
    fn create_optimized_runtime() -> Result<tokio::runtime::Runtime, String> {
        // Use single-threaded runtime only for Linux ARM64 (Raspberry Pi)
        // Keep multi-threaded for macOS (even on Apple Silicon) and other platforms
        if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
            // Runtime tuned for Raspberry Pi - small pool works better than fully single threaded
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| format!("Failed to create Raspberry Pi runtime: {}", e))
        } else {
            // Multi-threaded runtime for macOS, Windows, and other platforms
            println!(
                "🔧 Creating multi-threaded runtime for {}",
                if cfg!(target_os = "macos") {
                    "macOS"
                } else if cfg!(target_os = "windows") {
                    "Windows"
                } else {
                    "other platforms"
                }
            );
            tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))
        }
    }
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut app = Self {
            rt: Some(rt),
            api_client: ApiClient::new(),
            streaming_enabled: true,
            streaming_message: String::new(),
            streaming_active: false,
            streaming_promise: None,
            streaming_chat_id: None,
            streaming_message_base_len: 0,
            streaming_message_persisted: false,
            token_receiver: None,
            char_buffer: String::new(),
            char_display_index: 0,
            last_char_time: std::time::Instant::now(),
            token_counter: TokenCounter::default(),
            token_usage: TokenUsage::default(),
            last_request_provider: None,
            last_request_model: None,
            last_model_descriptor: None,
            ..Default::default()
        };
        app.load_data();

        if app.api_keys.get(app.current_provider).is_empty() {
            if let Some(provider_with_key) = ProviderKind::all()
                .into_iter()
                .find(|p| !app.api_keys.get(*p).is_empty())
            {
                app.current_provider = provider_with_key;
            }
        }

        // Apply theme to egui style
        cc.egui_ctx.style_mut(|style| {
            app.current_theme.apply_to_egui_style(style, app.font_size);
        });

        // Load models if we have an API key
        if !app.api_keys.get(app.current_provider).is_empty() {
            app.load_models();
        }

        app
    }

    fn load_data(&mut self) {
        self.api_keys = Storage::load_api_keys().unwrap_or_default();
        self.chats = Storage::load_chats().unwrap_or_default();
        self.projects = Storage::load_projects().unwrap_or_default();
        self.font_size = Storage::load_font_size().unwrap_or(14.0);
    }

    pub fn save_data(&self) {
        if let Err(e) = Storage::save_api_keys(&self.api_keys) {
            println!("❌ Failed to save API keys: {}", e);
        }
        if let Err(e) = Storage::save_chats(&self.chats) {
            println!("❌ Failed to save chats: {}", e);
        } else {
            println!(
                "✅ Chats saved successfully. Total chats: {}",
                self.chats.len()
            );
        }
        if let Err(e) = Storage::save_projects(&self.projects) {
            println!("❌ Failed to save projects: {}", e);
        }
        if let Err(e) = Storage::save_theme(&self.current_theme_type) {
            println!("❌ Failed to save theme: {}", e);
        }
        if let Err(e) = Storage::save_font_size(self.font_size) {
            println!("❌ Failed to save font size: {}", e);
        }
    }

    pub fn create_new_chat(&mut self) {
        let chat_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let chat = Chat {
            id: chat_id.clone(),
            title: "New Chat".to_string(),
            messages: Vec::new(),
            model: String::new(),
            provider: self.current_provider,
            project_id: self.current_project_id.clone(),
            created_at: now,
            updated_at: now,
        };

        self.chats.insert(chat_id.clone(), chat);
        self.current_chat_id = Some(chat_id);
        self.reset_token_usage();
        self.save_data();
    }

    pub fn load_models(&mut self) {
        if self.loading_models {
            return;
        }

        let provider = self.current_provider;
        let api_key = self.api_keys.get(provider).trim().to_string();

        if api_key.is_empty() {
            self.available_models.clear();
            self.current_model.clear();
            return;
        }

        self.loading_models = true;

        println!(
            "Loading models for {} with key prefix: {}",
            provider.display_name(),
            &api_key[..std::cmp::min(8, api_key.len())]
        );

        if let Some(rt) = &self.rt {
            let models_future = {
                let api_client = ApiClient::new();
                let api_key = api_key.clone();
                async move { api_client.load_models(provider, &api_key).await }
            };

            match rt.block_on(models_future) {
                Ok(models) => {
                    println!(
                        "Successfully loaded {} models for {}",
                        models.len(),
                        provider.display_name()
                    );
                    self.available_models = models;

                    let has_current = self
                        .available_models
                        .iter()
                        .any(|model| model.id == self.current_model);

                    if !has_current {
                        if let Some(first) = self.available_models.first() {
                            self.current_model = first.id.clone();
                        } else {
                            self.current_model.clear();
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error loading models for {}: {}",
                        provider.display_name(),
                        e
                    );
                    self.available_models.clear();
                    self.current_model.clear();
                }
            }
        }

        self.loading_models = false;
        self.refresh_token_usage_for_current_chat();
    }

    fn get_model_descriptor(
        &self,
        provider: ProviderKind,
        model_id: &str,
    ) -> Option<ModelDescriptor> {
        self.available_models
            .iter()
            .find(|model| model.id == model_id && model.provider == provider)
            .cloned()
            .or_else(|| {
                self.available_models
                    .iter()
                    .find(|model| model.id == model_id)
                    .cloned()
            })
    }

    fn context_length_for_model(
        &self,
        provider: ProviderKind,
        model_id: &str,
        descriptor: Option<&ModelDescriptor>,
    ) -> Option<usize> {
        if let Some(descriptor) = descriptor {
            if let Some(context_length) = descriptor.context_length {
                return Some(context_length);
            }
        }

        let normalized = model_id.to_lowercase();
        let fallback = match provider {
            ProviderKind::OpenAI | ProviderKind::OpenRouter => {
                if normalized.contains("3.5") {
                    16_385
                } else {
                    128_000
                }
            }
            ProviderKind::Anthropic => 200_000,
            ProviderKind::LMStudio | ProviderKind::Ollama => 4_096,
        };

        Some(fallback)
    }

    fn update_context_from_descriptor(
        &mut self,
        provider: ProviderKind,
        model_id: &str,
        descriptor: Option<&ModelDescriptor>,
    ) {
        self.token_usage.max_context_tokens =
            self.context_length_for_model(provider, model_id, descriptor);
    }

    pub(crate) fn refresh_token_usage_for_current_chat(&mut self) {
        let Some(chat_id) = &self.current_chat_id else {
            self.reset_token_usage();
            return;
        };

        let Some(chat) = self.chats.get(chat_id) else {
            self.reset_token_usage();
            return;
        };

        let provider = chat.provider;
        let model_id = chat.model.clone();
        let messages = chat.messages.clone();

        let descriptor = self
            .get_model_descriptor(provider, &model_id)
            .or_else(|| self.last_model_descriptor.clone());
        self.update_context_from_descriptor(provider, &model_id, descriptor.as_ref());
        self.last_model_descriptor = descriptor;
        self.last_request_provider = Some(provider);
        self.last_request_model = Some(model_id.clone());

        if messages.is_empty() {
            self.token_usage.prompt_tokens = 0;
            self.token_usage.completion_tokens = 0;
            self.token_usage.total_tokens = 0;
            return;
        }

        if let Some(last_assistant_index) = messages
            .iter()
            .rposition(|message| message.role.eq_ignore_ascii_case("assistant"))
        {
            let prompt_slice = &messages[..last_assistant_index];
            let prompt_tokens =
                self.token_counter
                    .count_prompt_tokens(provider, &model_id, prompt_slice);
            let completion_tokens = self.token_counter.count_completion_tokens(
                provider,
                &model_id,
                &messages[last_assistant_index].content,
            );

            self.token_usage.prompt_tokens = prompt_tokens;
            self.token_usage.completion_tokens = completion_tokens;
            self.token_usage.total_tokens = prompt_tokens + completion_tokens;
        } else {
            let prompt_tokens = self
                .token_counter
                .count_prompt_tokens(provider, &model_id, &messages);
            self.token_usage.prompt_tokens = prompt_tokens;
            self.token_usage.completion_tokens = 0;
            self.token_usage.total_tokens = prompt_tokens;
        }
    }

    pub(crate) fn reset_token_usage(&mut self) {
        self.token_usage = TokenUsage::default();
        self.last_request_provider = None;
        self.last_request_model = None;
        self.last_model_descriptor = None;
    }

    pub(crate) fn on_model_changed(&mut self) {
        if self.current_model.is_empty() {
            self.reset_token_usage();
            return;
        }

        let mut chat_updated = false;

        let provider = self.current_provider;
        let model_id = self.current_model.clone();

        if let Some(chat_id) = &self.current_chat_id {
            if let Some(chat) = self.chats.get_mut(chat_id) {
                chat.model = model_id.clone();
                chat.provider = provider;
                chat.updated_at = Utc::now();
                chat_updated = true;
            }
        }

        let descriptor = self
            .get_model_descriptor(provider, &model_id)
            .or_else(|| self.last_model_descriptor.clone());
        self.update_context_from_descriptor(provider, &model_id, descriptor.as_ref());
        self.last_model_descriptor = descriptor;
        self.refresh_token_usage_for_current_chat();

        if chat_updated {
            self.save_data();
        }
    }

    pub fn set_current_provider(&mut self, provider: ProviderKind) {
        if self.current_provider == provider {
            return;
        }

        self.current_provider = provider;
        self.current_model.clear();

        if self.api_keys.get(provider).trim().is_empty() {
            self.available_models.clear();
        } else {
            self.load_models();
        }
    }

    pub fn send_message(&mut self, _ctx: &egui::Context) {
        if self.message_input.trim().is_empty() {
            return;
        }

        if self.current_model.is_empty() {
            eprintln!("No model selected");
            return;
        }

        let provider = self.current_provider;
        let api_key = self.api_keys.get(provider).trim().to_string();

        if api_key.is_empty() {
            eprintln!("No API key provided for {}", provider.display_name());
            return;
        }

        if self.sending_message || self.streaming_active {
            println!(
                "Prevented duplicate send: sending={}, streaming={}",
                self.sending_message, self.streaming_active
            );
            return; // Already sending a message or streaming
        }

        println!(
            "Starting new message send (streaming enabled: {}) on {}",
            self.streaming_enabled,
            if cfg!(target_os = "macos") {
                "macOS"
            } else if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
                "Raspberry Pi"
            } else {
                "other platform"
            }
        );

        // Create chat if needed
        if self.current_chat_id.is_none() {
            self.create_new_chat();
        }

        let chat_id = self.current_chat_id.as_ref().unwrap().clone();

        // Process attachments
        let mut attachments = Vec::new();
        for file_path in &self.selected_attachments {
            match AttachmentProcessor::process_file(file_path) {
                Ok(attachment) => {
                    attachments.push(attachment);
                }
                Err(e) => {
                    eprintln!("Error processing attachment {}: {}", file_path, e);
                    // Could show error to user in UI instead
                }
            }
        }

        let user_message = Message {
            role: "user".to_string(),
            content: self.message_input.clone(),
            timestamp: Utc::now(),
            attachments,
            provider: None,
            model: None,
        };

        // Add user message immediately - this will show in UI right away
        if let Some(chat) = self.chats.get_mut(&chat_id) {
            chat.messages.push(user_message.clone());
            chat.updated_at = Utc::now();
            chat.model = self.current_model.clone();
            chat.provider = provider;

            // Update title if it's the first message
            if chat.title == "New Chat" {
                let title = user_message.content.chars().take(50).collect::<String>();
                chat.title = if title.len() == 50 {
                    format!("{}...", title)
                } else {
                    title
                };
            }
        }

        // Clear input and attachments, set sending state
        self.message_input.clear();
        self.selected_attachments.clear();
        self.save_data();

        // Start async API call
        let model = self.current_model.clone();
        let history = self.chats[&chat_id].messages.clone();

        let descriptor = self
            .get_model_descriptor(provider, &model)
            .or_else(|| self.last_model_descriptor.clone());
        self.last_model_descriptor = descriptor.clone();
        self.last_request_provider = Some(provider);
        self.last_request_model = Some(model.clone());
        self.update_context_from_descriptor(provider, &model, descriptor.as_ref());

        let prompt_tokens = self
            .token_counter
            .count_prompt_tokens(provider, &model, &history);
        self.token_usage.prompt_tokens = prompt_tokens;
        self.token_usage.completion_tokens = 0;
        self.token_usage.total_tokens = prompt_tokens;

        if self.streaming_enabled {
            // Use real-time streaming with character-level smoothing
            self.streaming_active = true;
            self.stream_completed = false;
            self.streaming_result = None;
            self.streaming_message.clear();
            self.char_buffer.clear();
            self.char_display_index = 0;
            self.last_char_time = std::time::Instant::now();
            self.streaming_chat_id = Some(chat_id.clone());
            self.streaming_message_base_len = self.chats[&chat_id].messages.len();
            self.streaming_message_persisted = false;

            // Create channel for real-time token communication
            let (token_sender, token_receiver) = mpsc::channel();
            self.token_receiver = Some(token_receiver);

            let stream_history = history.clone();
            let promise = Promise::spawn_thread("streaming_api_call", move || {
                let rt = match Self::create_optimized_runtime() {
                    Ok(rt) => rt,
                    Err(e) => {
                        println!("❌ Failed to create optimized runtime: {}", e);
                        return Err(e);
                    }
                };

                rt.block_on(async move {
                    use futures_util::StreamExt;

                    let api_client = ApiClient::new();
                    match api_client
                        .stream_chat_completion(provider, &api_key, &model, stream_history)
                        .await
                    {
                        Ok(stream) => {
                            let mut full_response = String::new();
                            let mut pinned_stream = Box::pin(stream);

                            while let Some(token_result) = pinned_stream.next().await {
                                match token_result {
                                    Ok(token) => {
                                        full_response.push_str(&token);

                                        // Send token to UI immediately for real-time display
                                        if token_sender.send(token).is_err() {
                                            // UI has been dropped, stop streaming
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        return Err(format!("Token stream error: {}", e));
                                    }
                                }
                            }

                            Ok(full_response)
                        }
                        Err(e) => Err(format!("Streaming API request failed: {}", e)),
                    }
                })
            });

            self.streaming_promise = Some(promise);
        } else {
            // Use traditional non-streaming
            self.sending_message = true;
            self.streaming_chat_id = None;
            self.streaming_message_base_len = 0;
            self.streaming_message_persisted = false;

            let promise = Promise::spawn_thread("api_call", move || {
                let rt = match Self::create_optimized_runtime() {
                    Ok(rt) => rt,
                    Err(e) => {
                        println!("❌ Failed to create optimized runtime: {}", e);
                        return Err(e);
                    }
                };

                rt.block_on(async move {
                    let api_client = ApiClient::new();
                    match api_client
                        .send_chat_message(provider, &api_key, &model, history)
                        .await
                    {
                        Ok(response) => Ok(response),
                        Err(e) => Err(format!("API request failed: {}", e)),
                    }
                })
            });

            self.api_promise = Some(promise);
        }
    }

    pub fn delete_chat(&mut self, chat_id: &str) {
        self.chats.remove(chat_id);
        if self.current_chat_id.as_ref() == Some(&chat_id.to_string()) {
            self.current_chat_id = None;
            self.reset_token_usage();
        }
        self.save_data();
    }

    pub fn export_current_chat(&self) {
        if let Some(chat_id) = &self.current_chat_id {
            if let Some(chat) = self.chats.get(chat_id) {
                let markdown = Exporter::generate_chat_markdown(chat);
                let filename = format!("{}.md", chat.title.replace(" ", "_"));
                Exporter::export_to_file(&markdown, &filename);
            }
        }
    }

    pub fn handle_streaming_tokens(&mut self, ctx: &egui::Context) {
        if let Some(receiver) = &self.token_receiver {
            let mut received_new_tokens = false;

            // 1. Collect new tokens into buffer
            while let Ok(token) = receiver.try_recv() {
                self.char_buffer.push_str(&token);
                received_new_tokens = true;
            }

            // 2. Smooth character-by-character release from buffer
            let now = std::time::Instant::now();
            let char_interval = std::time::Duration::from_millis(15); // ~67 chars/sec for smooth effect

            if now.duration_since(self.last_char_time) >= char_interval
                && self.char_display_index < self.char_buffer.len()
            {
                // Add next character to display
                if let Some(next_char) = self.char_buffer.chars().nth(self.char_display_index) {
                    self.streaming_message.push(next_char);
                    self.char_display_index += 1;
                    self.last_char_time = now;

                    ctx.request_repaint();
                }
            }

            // 3. Continue updating if there are more characters to display
            if self.char_display_index < self.char_buffer.len() {
                ctx.request_repaint_after(char_interval);
            }

            // 4. Request repaint if we got new tokens (for immediate buffer effect)
            if received_new_tokens {
                ctx.request_repaint();

                if let Some(provider) = self.last_request_provider {
                    let model_id = self
                        .last_request_model
                        .as_deref()
                        .unwrap_or(&self.current_model);
                    let completion_tokens = self.token_counter.count_completion_tokens(
                        provider,
                        model_id,
                        &self.char_buffer,
                    );
                    self.token_usage.completion_tokens = completion_tokens;
                    self.token_usage.total_tokens =
                        self.token_usage.prompt_tokens + completion_tokens;
                }
            }
        }
    }

    fn persist_streaming_result(&mut self, result: &Result<String, String>) {
        if self.streaming_message_persisted {
            return; // Already saved
        }

        let target_chat_id = self
            .streaming_chat_id
            .clone()
            .or_else(|| self.current_chat_id.clone());

        let Some(chat_id) = target_chat_id else {
            eprintln!("Streaming completed but no chat ID was available to store the response.");
            return;
        };

        let final_content = match result {
            Ok(full_response) => {
                let content = if !self.char_buffer.is_empty() {
                    self.char_buffer.clone()
                } else if !self.streaming_message.is_empty() {
                    self.streaming_message.clone()
                } else {
                    full_response.clone()
                };

                content
            }
            Err(e) => {
                eprintln!("Streaming error: {}", e);

                let error_content = if e.contains("Rate limit exceeded") {
                    "Rate limit exceeded. Please wait a few minutes before trying again, or try a different model.".to_string()
                } else if e.contains("API Error 429") {
                    "Too many requests. Please wait before sending another message.".to_string()
                } else {
                    format!("Streaming error: {}", e)
                };

                error_content
            }
        };

        let persisted = if let Some(chat) = self.chats.get_mut(&chat_id) {
            let target_index = self.streaming_message_base_len;
            let message = Message {
                role: "assistant".to_string(),
                content: final_content.clone(),
                timestamp: Utc::now(),
                attachments: Vec::new(),
                provider: self.last_request_provider,
                model: self.last_request_model.clone(),
            };

            let stored = if chat.messages.len() == target_index {
                chat.messages.push(message);
                true
            } else if chat.messages.len() > target_index {
                chat.messages[target_index] = message;
                true
            } else {
                // Unexpected state; append to avoid data loss
                chat.messages.push(message);
                true
            };

            if stored {
                chat.updated_at = Utc::now();
            }

            stored
        } else {
            eprintln!("Failed to find chat with ID: {}", chat_id);
            return;
        };

        if persisted {
            self.save_data();
            self.streaming_message_persisted = true;
            self.refresh_token_usage_for_current_chat();
        }
    }

    pub fn handle_streaming_response(&mut self) {
        // First, check if we got a new result from the promise
        if let Some(promise) = &self.streaming_promise {
            if let Some(result) = promise.ready() {
                self.streaming_result = Some(result.clone());
                self.stream_completed = true;
            }
        }

        // Then, handle the stored result if we have one
        if let Some(result_clone) = self.streaming_result.clone() {
            if !self.streaming_message_persisted {
                self.persist_streaming_result(&result_clone);
            }

            let total_chars = self.char_buffer.chars().count();
            let still_displaying = self.char_display_index < total_chars;

            if still_displaying {
                return;
            }

            self.streaming_active = false;
            self.token_receiver = None; // Clean up the token receiver
            self.streaming_message.clear();
            self.char_buffer.clear();
            self.char_display_index = 0;
            self.stream_completed = false;
            self.streaming_result = None;
            self.streaming_chat_id = None;
            self.streaming_message_base_len = 0;
            self.streaming_message_persisted = false;
            self.streaming_promise = None;
        }
    }

    pub fn handle_api_response(&mut self) {
        if let Some(promise) = &self.api_promise {
            if let Some(result) = promise.ready() {
                // Handle traditional non-streaming completion
                self.sending_message = false;

                if let Some(current_chat_id) = &self.current_chat_id {
                    match result {
                        Ok(assistant_response) => {
                            let assistant_message = Message {
                                role: "assistant".to_string(),
                                content: assistant_response.clone(),
                                timestamp: Utc::now(),
                                attachments: Vec::new(),
                                provider: self.last_request_provider,
                                model: self.last_request_model.clone(),
                            };

                            if let Some(chat) = self.chats.get_mut(current_chat_id) {
                                chat.messages.push(assistant_message);
                                chat.updated_at = Utc::now();
                            }
                        }
                        Err(e) => {
                            let error_message = Message {
                                role: "assistant".to_string(),
                                content: format!("Error: {}", e),
                                timestamp: Utc::now(),
                                attachments: Vec::new(),
                                provider: self.last_request_provider,
                                model: self.last_request_model.clone(),
                            };

                            if let Some(chat) = self.chats.get_mut(current_chat_id) {
                                chat.messages.push(error_message);
                            }
                        }
                    }

                    self.save_data();
                    self.refresh_token_usage_for_current_chat();
                }

                self.api_promise = None;
            }
        }
    }

    pub fn change_theme(&mut self, ctx: &egui::Context, theme_type: ThemeType) {
        if self.current_theme_type != theme_type {
            self.current_theme_type = theme_type;
            self.current_theme = Theme::from_theme_type(&self.current_theme_type);

            // Apply new theme to egui style
            ctx.style_mut(|style| {
                self.current_theme
                    .apply_to_egui_style(style, self.font_size);
            });

            // Save the theme preference
            self.save_data();
        }
    }

    pub fn open_file_picker(&mut self) {
        // Start async file picker without blocking
        let promise = Promise::spawn_thread("file_picker", move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let files = rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                    .add_filter("PDFs", &["pdf"])
                    .add_filter("Text files", &["txt", "md", "json", "xml", "csv", "log"])
                    .add_filter(
                        "All supported",
                        &[
                            "png", "jpg", "jpeg", "gif", "webp", "bmp", "pdf", "txt", "md", "json",
                            "xml", "csv", "log",
                        ],
                    )
                    .pick_files()
                    .await;

                // Convert to file paths
                files.map(|files| {
                    files
                        .into_iter()
                        .map(|file| file.path().to_string_lossy().to_string())
                        .filter(|path| {
                            crate::attachments::AttachmentProcessor::is_supported_file(path)
                        })
                        .collect::<Vec<String>>()
                })
            })
        });

        self.file_picker_promise = Some(promise);
    }

    pub fn handle_file_picker_response(&mut self) {
        if let Some(promise) = &self.file_picker_promise {
            if let Some(result) = promise.ready() {
                if let Some(file_paths) = result {
                    self.selected_attachments.extend(file_paths.clone());
                }
                self.file_picker_promise = None;
            }
        }
    }

    // Project management methods
    pub fn create_project(&mut self, name: String) {
        if name.trim().is_empty() {
            return;
        }

        let project = Project::new(name.trim().to_string());
        let project_id = project.id.clone();
        self.projects.insert(project_id.clone(), project);
        self.current_project_id = Some(project_id);
        self.save_data();
    }

    pub fn rename_project(&mut self, project_id: String, new_name: String) {
        if new_name.trim().is_empty() {
            return;
        }

        if let Some(project) = self.projects.get_mut(&project_id) {
            project.name = new_name.trim().to_string();
            project.updated_at = Utc::now();
            self.save_data();
        }
    }

    pub fn delete_project(&mut self, project_id: &str) {
        // Unassign all chats from this project
        for chat in self.chats.values_mut() {
            if chat.project_id.as_ref() == Some(&project_id.to_string()) {
                chat.project_id = None;
            }
        }

        // Remove the project
        self.projects.remove(project_id);

        // Clear current project if it was deleted
        if self.current_project_id.as_ref() == Some(&project_id.to_string()) {
            self.current_project_id = None;
        }

        self.save_data();
    }

    pub fn move_chat_to_project(&mut self, chat_id: &str, project_id: Option<String>) {
        if let Some(chat) = self.chats.get_mut(chat_id) {
            chat.project_id = project_id;
            chat.updated_at = Utc::now();
            self.save_data();
        }
    }

    pub fn get_filtered_chats(&self) -> Vec<&Chat> {
        let mut chats: Vec<_> = self.chats.values()
            .filter(|chat| {
                if let Some(current_project_id) = &self.current_project_id {
                    // Show only chats in current project
                    chat.project_id.as_ref() == Some(current_project_id)
                } else {
                    // "All Chats" view - show all chats
                    true
                }
            })
            .collect();

        chats.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        chats
    }
}
