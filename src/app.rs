use crate::api::ApiClient;
use crate::export::Exporter;
use crate::storage::Storage;
use crate::theme::{Theme, ThemeType};
use crate::types::{Chat, ChatMap, FolderMap, Message, OpenRouterModel};
use chrono::Utc;
use eframe::egui;
use poll_promise::Promise;

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
            api_client: ApiClient::new(),
        }
    }
}

impl OverlookApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut app = Self {
            rt: Some(rt),
            api_client: ApiClient::new(),
            ..Default::default()
        };
        app.load_data();

        // Apply theme to egui style
        cc.egui_ctx.style_mut(|style| {
            app.current_theme.apply_to_egui_style(style);
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
    }

    pub fn save_data(&self) {
        let _ = Storage::save_api_key(&self.api_key);
        let _ = Storage::save_chats(&self.chats);
        let _ = Storage::save_folders(&self.folders);
        let _ = Storage::save_theme(&self.current_theme_type);
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

        println!("Loading models with API key: {}...", &api_key[..std::cmp::min(8, api_key.len())]);

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

        if self.sending_message {
            return; // Already sending a message
        }

        // Create chat if needed
        if self.current_chat_id.is_none() {
            self.create_new_chat();
        }

        let chat_id = self.current_chat_id.as_ref().unwrap().clone();
        let user_message = Message {
            role: "user".to_string(),
            content: self.message_input.clone(),
            timestamp: Utc::now(),
        };

        // Add user message immediately - this will show in UI right away
        if let Some(chat) = self.chats.get_mut(&chat_id) {
            chat.messages.push(user_message.clone());
            chat.updated_at = Utc::now();

            // Update title if it's the first message
            if chat.title == "New Chat" {
                let title = user_message.content.chars().take(50).collect::<String>();
                chat.title = if title.len() == 50 { format!("{}...", title) } else { title };
            }
        }

        // Clear input and set sending state
        self.message_input.clear();
        self.sending_message = true;
        self.save_data();

        // Start async API call
        let api_key = self.api_key.clone();
        let model = self.current_model.clone();
        let messages: Vec<_> = self.chats[&chat_id].messages.iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

        let promise = Promise::spawn_thread("api_call", move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let api_client = ApiClient::new();
                match api_client.send_chat_message(&api_key, &model, messages).await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(format!("API request failed: {}", e)),
                }
            })
        });

        self.api_promise = Some(promise);
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

    pub fn handle_api_response(&mut self) {
        if let Some(promise) = &self.api_promise {
            if let Some(result) = promise.ready() {
                self.sending_message = false;

                if let Some(current_chat_id) = &self.current_chat_id {
                    match result {
                        Ok(assistant_response) => {
                            let assistant_message = Message {
                                role: "assistant".to_string(),
                                content: assistant_response.clone(),
                                timestamp: Utc::now(),
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
                self.current_theme.apply_to_egui_style(style);
            });

            // Save the theme preference
            self.save_data();
        }
    }
}