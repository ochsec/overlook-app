use crate::api::ApiClient;
use crate::attachments::AttachmentProcessor;
use crate::export::Exporter;
use crate::storage::Storage;
use crate::theme::{Theme, ThemeType};
use crate::types::{Chat, ChatMap, FolderMap, Message, OpenRouterModel};
use chrono::Utc;
use eframe::egui;
use poll_promise::Promise;
use std::sync::mpsc;

pub struct OverlookApp {
    pub api_key: String,
    pub current_chat_id: Option<String>,
    pub current_model: String,
    pub message_input: String,
    pub chats: ChatMap,
    pub folders: FolderMap,
    pub available_models: Vec<OpenRouterModel>,
    pub loading_models: bool,
    pub sending_message: bool,
    pub show_folder_modal: bool,
    pub new_folder_name: String,
    pub selected_folder_id: String,
    pub show_export_modal: bool,
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
    api_client: ApiClient,
}

impl Default for OverlookApp {
    fn default() -> Self {
        let theme_type = Storage::load_theme().unwrap_or_default();
        let theme = Theme::from_theme_type(&theme_type);
        Self {
            api_key: String::default(),
            current_chat_id: None,
            current_model: String::default(),
            message_input: String::default(),
            chats: ChatMap::default(),
            folders: FolderMap::default(),
            available_models: Vec::default(),
            loading_models: bool::default(),
            sending_message: bool::default(),
            show_folder_modal: bool::default(),
            new_folder_name: String::default(),
            selected_folder_id: String::default(),
            show_export_modal: bool::default(),
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
            // Single-threaded runtime for Raspberry Pi - much more efficient
            println!("🔧 Creating single-threaded runtime for Raspberry Pi (ARM64 Linux)");
            tokio::runtime::Builder::new_current_thread()
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
            ..Default::default()
        };
        app.load_data();

        // Apply theme to egui style
        cc.egui_ctx.style_mut(|style| {
            app.current_theme.apply_to_egui_style(style, app.font_size);
        });

        // Load models if we have an API key
        if !app.api_key.is_empty() {
            app.load_models();
        }

        app
    }

    fn load_data(&mut self) {
        self.api_key = Storage::load_api_key().unwrap_or_default();
        self.chats = Storage::load_chats().unwrap_or_default();
        self.folders = Storage::load_folders().unwrap_or_default();
        self.font_size = Storage::load_font_size().unwrap_or(14.0);
    }

    pub fn save_data(&self) {
        if let Err(e) = Storage::save_api_key(&self.api_key) {
            println!("❌ Failed to save API key: {}", e);
        }
        if let Err(e) = Storage::save_chats(&self.chats) {
            println!("❌ Failed to save chats: {}", e);
        } else {
            println!(
                "✅ Chats saved successfully. Total chats: {}",
                self.chats.len()
            );
        }
        if let Err(e) = Storage::save_folders(&self.folders) {
            println!("❌ Failed to save folders: {}", e);
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
            folder: None,
            created_at: now,
            updated_at: now,
        };

        self.chats.insert(chat_id.clone(), chat);
        self.current_chat_id = Some(chat_id);
        self.save_data();
    }

    pub fn load_models(&mut self) {
        if self.api_key.is_empty() || self.loading_models {
            return;
        }

        self.loading_models = true;
        let api_key = self.api_key.clone();

        println!(
            "Loading models with API key: {}...",
            &api_key[..std::cmp::min(8, api_key.len())]
        );

        if let Some(rt) = &self.rt {
            let api_client = ApiClient::new();
            let models_future = async move {
                match api_client.load_models(&api_key).await {
                    Ok(models) => {
                        println!("Successfully loaded {} models", models.len());
                        Ok(models)
                    }
                    Err(e) => {
                        eprintln!("Error loading models: {}", e);
                        Err(format!("Failed to load models: {}", e))
                    }
                }
            };

            match rt.block_on(models_future) {
                Ok(models) => {
                    self.available_models = models;
                    println!("Models loaded successfully!");
                }
                Err(e) => {
                    eprintln!("Error loading models: {}", e);
                    self.available_models.clear();
                }
            }
        }

        self.loading_models = false;
    }

    pub fn send_message(&mut self, _ctx: &egui::Context) {
        if self.message_input.trim().is_empty() {
            return;
        }

        if self.current_model.is_empty() {
            eprintln!("No model selected");
            return;
        }

        if self.api_key.is_empty() {
            eprintln!("No API key provided");
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
        };

        // Add user message immediately - this will show in UI right away
        if let Some(chat) = self.chats.get_mut(&chat_id) {
            chat.messages.push(user_message.clone());
            chat.updated_at = Utc::now();

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
        let api_key = self.api_key.clone();
        let model = self.current_model.clone();
        let messages: Vec<_> = self.chats[&chat_id]
            .messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

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
                        .stream_chat_completion(&api_key, &model, messages)
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
                        .send_chat_message(&api_key, &model, messages)
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
                chat.model = self.current_model.clone();
            }

            stored
        } else {
            eprintln!("Failed to find chat with ID: {}", chat_id);
            return;
        };

        if persisted {
            self.save_data();
            self.streaming_message_persisted = true;
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
                            };

                            if let Some(chat) = self.chats.get_mut(current_chat_id) {
                                chat.messages.push(assistant_message);
                                chat.updated_at = Utc::now();
                                chat.model = self.current_model.clone();
                            }
                        }
                        Err(e) => {
                            let error_message = Message {
                                role: "assistant".to_string(),
                                content: format!("Error: {}", e),
                                timestamp: Utc::now(),
                                attachments: Vec::new(),
                            };

                            if let Some(chat) = self.chats.get_mut(current_chat_id) {
                                chat.messages.push(error_message);
                            }
                        }
                    }

                    self.save_data();
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
}
