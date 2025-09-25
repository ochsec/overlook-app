use eframe::egui;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use poll_promise::Promise;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Chat {
    id: String,
    title: String,
    messages: Vec<Message>,
    model: String,
    folder: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenRouterModel {
    id: String,
    name: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Default)]
struct OverlookApp {
    api_key: String,
    current_chat_id: Option<String>,
    current_model: String,
    message_input: String,
    chats: HashMap<String, Chat>,
    folders: HashMap<String, String>,
    available_models: Vec<OpenRouterModel>,
    loading_models: bool,
    sending_message: bool,
    show_folder_modal: bool,
    new_folder_name: String,
    selected_folder_id: String,
    show_export_modal: bool,
    rt: Option<tokio::runtime::Runtime>,
    api_promise: Option<Promise<Result<String, String>>>,
}

impl OverlookApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut app = Self {
            rt: Some(rt),
            ..Default::default()
        };
        app.load_data();

        // Load models if we have an API key
        if !app.api_key.is_empty() {
            app.load_models();
        }

        app
    }

    fn load_data(&mut self) {
        // Load from persistent storage (similar to localStorage in the original)
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");

            // Load API key
            if let Ok(key) = std::fs::read_to_string(app_dir.join("api_key.txt")) {
                self.api_key = key.trim().to_string();
            }

            // Load chats
            if let Ok(chats_data) = std::fs::read_to_string(app_dir.join("chats.json")) {
                if let Ok(chats) = serde_json::from_str::<HashMap<String, Chat>>(&chats_data) {
                    self.chats = chats;
                }
            }

            // Load folders
            if let Ok(folders_data) = std::fs::read_to_string(app_dir.join("folders.json")) {
                if let Ok(folders) = serde_json::from_str::<HashMap<String, String>>(&folders_data) {
                    self.folders = folders;
                }
            }
        }
    }

    fn save_data(&self) {
        if let Some(data_dir) = dirs::config_dir() {
            let app_dir = data_dir.join("overlook");
            let _ = std::fs::create_dir_all(&app_dir);

            // Save API key
            let _ = std::fs::write(app_dir.join("api_key.txt"), &self.api_key);

            // Save chats
            if let Ok(chats_json) = serde_json::to_string_pretty(&self.chats) {
                let _ = std::fs::write(app_dir.join("chats.json"), chats_json);
            }

            // Save folders
            if let Ok(folders_json) = serde_json::to_string_pretty(&self.folders) {
                let _ = std::fs::write(app_dir.join("folders.json"), folders_json);
            }
        }
    }

    fn create_new_chat(&mut self) {
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

    fn load_models(&mut self) {
        if self.api_key.is_empty() || self.loading_models {
            return;
        }

        self.loading_models = true;
        let api_key = self.api_key.clone();

        println!("Loading models with API key: {}...", &api_key[..std::cmp::min(8, api_key.len())]);

        if let Some(rt) = &self.rt {
            let models_future = async move {
                let client = reqwest::Client::new();
                let response = client
                    .get("https://openrouter.ai/api/v1/models")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("HTTP-Referer", "https://github.com/your-username/overlook")
                    .header("X-Title", "Overlook Chat")
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        let status = resp.status();
                        println!("Models API response status: {}", status);

                        if !status.is_success() {
                            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                            return Err(format!("API Error {}: {}", status, error_text));
                        }

                        match resp.json::<ModelsResponse>().await {
                            Ok(models_response) => {
                                println!("Successfully loaded {} models", models_response.data.len());
                                Ok(models_response.data)
                            },
                            Err(e) => Err(format!("Failed to parse models response: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Failed to fetch models: {}", e)),
                }
            };

            match rt.block_on(models_future) {
                Ok(models) => {
                    self.available_models = models;
                    println!("Models loaded successfully!");
                }
                Err(e) => {
                    eprintln!("Error loading models: {}", e);
                    // Clear models on error
                    self.available_models.clear();
                }
            }
        }

        self.loading_models = false;
    }

    fn send_message(&mut self, _ctx: &egui::Context) {
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
            let client = reqwest::Client::new();
            let request_body = serde_json::json!({
                "model": model,
                "messages": messages,
                "temperature": 0.7
            });

            let response = client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .header("HTTP-Referer", "https://github.com/your-username/overlook")
                .header("X-Title", "Overlook Chat")
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    match resp.json::<serde_json::Value>().await {
                        Ok(data) => {
                            if let Some(choice) = data["choices"].as_array().and_then(|arr| arr.first()) {
                                if let Some(content) = choice["message"]["content"].as_str() {
                                    return Ok(content.to_string());
                                }
                            }
                            Err("Invalid response format".to_string())
                        }
                        Err(e) => Err(format!("Failed to parse response: {}", e)),
                    }
                }
                Err(e) => Err(format!("API request failed: {}", e)),
            }
            })
        });

        self.api_promise = Some(promise);
    }

    fn delete_chat(&mut self, chat_id: &str) {
        self.chats.remove(chat_id);
        if self.current_chat_id.as_ref() == Some(&chat_id.to_string()) {
            self.current_chat_id = None;
        }
        self.save_data();
    }

    fn export_current_chat(&self) {
        if let Some(chat_id) = &self.current_chat_id {
            if let Some(chat) = self.chats.get(chat_id) {
                let markdown = self.generate_chat_markdown(chat);
                let filename = format!("{}.md", chat.title.replace(" ", "_"));

                if let Some(downloads_dir) = dirs::download_dir() {
                    let file_path = downloads_dir.join(filename);
                    let _ = std::fs::write(file_path, markdown);
                }
            }
        }
    }

    fn generate_chat_markdown(&self, chat: &Chat) -> String {
        let mut markdown = format!("# {}\n\n", chat.title);
        markdown.push_str(&format!("**Model:** {}\n", chat.model));
        markdown.push_str(&format!("**Created:** {}\n", chat.created_at.format("%Y-%m-%d %H:%M:%S")));
        markdown.push_str(&format!("**Updated:** {}\n\n", chat.updated_at.format("%Y-%m-%d %H:%M:%S")));

        for message in &chat.messages {
            let role = if message.role == "user" { "User" } else { "Assistant" };
            markdown.push_str(&format!("## {}\n\n{}\n\n", role, message.content));
        }

        markdown
    }
}

impl eframe::App for OverlookApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for completed API promise
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
        egui::SidePanel::left("sidebar").resizable(true).min_width(300.0).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("OpenRouter Chat");
                ui.separator();

                // API Key input
                ui.label("API Key:");
                let response = ui.text_edit_singleline(&mut self.api_key);
                if response.changed() {
                    self.save_data();
                    self.load_models();
                }

                ui.separator();

                // New Chat button
                if ui.button("+ New Chat").clicked() {
                    self.create_new_chat();
                }

                ui.separator();

                // Chats list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut chats_vec: Vec<_> = self.chats.iter().map(|(_, chat)| chat.clone()).collect();
                    chats_vec.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                    let mut chat_to_delete: Option<String> = None;
                    let mut chat_to_select: Option<String> = None;

                    for chat in chats_vec {
                        let is_selected = self.current_chat_id.as_ref() == Some(&chat.id);

                        ui.horizontal(|ui| {
                            let button = egui::Button::new(&chat.title)
                                .fill(if is_selected {
                                    egui::Color32::from_rgb(0, 102, 204)
                                } else {
                                    egui::Color32::TRANSPARENT
                                });

                            if ui.add_sized([200.0, 30.0], button).clicked() {
                                chat_to_select = Some(chat.id.clone());
                            }

                            if ui.small_button("🗑").clicked() {
                                chat_to_delete = Some(chat.id.clone());
                            }
                        });
                    }

                    // Handle actions outside the loop
                    if let Some(chat_id) = chat_to_select {
                        self.current_chat_id = Some(chat_id.clone());
                        if let Some(current_chat) = self.chats.get(&chat_id) {
                            self.current_model = current_chat.model.clone();
                        }
                    }

                    if let Some(chat_id) = chat_to_delete {
                        self.delete_chat(&chat_id);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label("Model:");

                    let combo_text = if self.loading_models {
                        "Loading models..."
                    } else if self.available_models.is_empty() {
                        if self.api_key.is_empty() {
                            "Enter API key first"
                        } else {
                            "No models loaded"
                        }
                    } else if self.current_model.is_empty() {
                        "Select a model..."
                    } else {
                        // Find the model name for display
                        self.available_models
                            .iter()
                            .find(|m| m.id == self.current_model)
                            .map(|m| m.name.as_str())
                            .unwrap_or(&self.current_model)
                    };

                    egui::ComboBox::from_id_source("model_selector")
                        .selected_text(combo_text)
                        .show_ui(ui, |ui| {
                            for model in &self.available_models {
                                ui.selectable_value(&mut self.current_model, model.id.clone(), &model.name);
                            }
                        });

                    // Refresh models button
                    if ui.small_button("🔄").clicked() && !self.loading_models {
                        self.load_models();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📥 Export").clicked() {
                            self.show_export_modal = true;
                        }

                        if ui.button("📁 Organize").clicked() {
                            self.show_folder_modal = true;
                        }
                    });
                });

                ui.separator();

                // Chat area - takes up most of the space
                let available_height = ui.available_height() - 120.0; // Reserve space for input
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(chat_id) = &self.current_chat_id {
                            if let Some(chat) = self.chats.get(chat_id) {
                                for message in &chat.messages {
                                    ui.add_space(5.0);

                                    if message.role == "user" {
                                        // User message - right aligned with blue background
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            ui.add_space(20.0); // Right margin

                                            let mut job = egui::text::LayoutJob::default();
                                            job.append(
                                                &message.content,
                                                0.0,
                                                egui::TextFormat {
                                                    color: egui::Color32::WHITE,
                                                    background: egui::Color32::from_rgb(0, 102, 204),
                                                    ..Default::default()
                                                },
                                            );

                                            ui.allocate_ui_with_layout(
                                                [ui.available_width() * 0.7, 0.0].into(),
                                                egui::Layout::top_down(egui::Align::LEFT),
                                                |ui| {
                                                    ui.add(
                                                        egui::Label::new(job)
                                                            .wrap()
                                                            .selectable(true)
                                                    );
                                                }
                                            );
                                        });
                                    } else {
                                        // Assistant message - left aligned with gray background
                                        ui.horizontal(|ui| {
                                            ui.add_space(20.0); // Left margin

                                            let mut job = egui::text::LayoutJob::default();
                                            job.append(
                                                &message.content,
                                                0.0,
                                                egui::TextFormat {
                                                    color: egui::Color32::WHITE,
                                                    background: egui::Color32::from_rgb(45, 45, 45),
                                                    ..Default::default()
                                                },
                                            );

                                            ui.allocate_ui_with_layout(
                                                [ui.available_width() * 0.8, 0.0].into(),
                                                egui::Layout::top_down(egui::Align::LEFT),
                                                |ui| {
                                                    ui.add(
                                                        egui::Label::new(job)
                                                            .wrap()
                                                            .selectable(true)
                                                    );
                                                }
                                            );
                                        });
                                    }
                                    ui.add_space(8.0);
                                }

                                if self.sending_message {
                                    ui.horizontal(|ui| {
                                        ui.add_space(20.0);
                                        ui.label(egui::RichText::new("AI is typing...")
                                            .color(egui::Color32::GRAY)
                                            .italics());
                                    });
                                }
                            }
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(100.0);
                                ui.heading("Welcome to Overlook Chat");
                                ui.add_space(10.0);
                                ui.label("Select a model and start chatting, or create a new conversation.");
                            });
                        }
                    });

                ui.separator();

                // Message input area - larger and more prominent
                ui.vertical(|ui| {
                    ui.add_space(5.0);

                    // Input area with better sizing
                    ui.horizontal(|ui| {
                        // Large text input that takes most of the width
                        let text_edit_response = ui.add_sized(
                            [ui.available_width() - 80.0, 60.0], // Make input much taller
                            egui::TextEdit::multiline(&mut self.message_input)
                                .hint_text("Type your message here... (Shift+Enter for new line, Enter to send)")
                                .desired_rows(3)
                        );

                        // Handle Enter key to send (but not Shift+Enter)
                        if text_edit_response.has_focus() && ui.input(|i| {
                            i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                        }) {
                            self.send_message(ctx);
                        }

                        // Send button
                        ui.vertical(|ui| {
                            ui.add_space(15.0); // Center the button vertically
                            let send_btn = ui.add_sized(
                                [70.0, 30.0],
                                egui::Button::new(if self.sending_message { "Sending..." } else { "Send" })
                            );

                            if send_btn.clicked() && !self.sending_message {
                                self.send_message(ctx);
                            }
                        });
                    });

                    ui.add_space(5.0);
                });
            });
        });

        // Export modal
        if self.show_export_modal {
            egui::Window::new("Export Chat")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Export Current Chat").clicked() {
                            self.export_current_chat();
                            self.show_export_modal = false;
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_export_modal = false;
                        }
                    });
                });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Overlook - OpenRouter Chat",
        options,
        Box::new(|cc| Ok(Box::new(OverlookApp::new(cc)))),
    )
}