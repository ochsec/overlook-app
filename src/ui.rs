use crate::app::{OverlookApp, ViewMode};
use crate::theme::{Theme, ThemeType};
use crate::types::ProviderKind;
use eframe::egui;

impl eframe::App for OverlookApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for streaming tokens
        self.handle_streaming_tokens(ctx);

        // Check for completed streaming response
        self.handle_streaming_response();

        // Check for completed API promise
        self.handle_api_response();

        // Check for completed file picker
        self.handle_file_picker_response();

        // Check for completed models loading
        self.handle_models_response();

        self.render_menu_bar(ctx);
        self.render_sidebar(ctx);
        self.render_main_content(ctx);
        self.render_modals(ctx);

        // Request repaint only when we actually have streaming updates
        // (The handle_streaming_tokens method will request repaints when tokens arrive)
    }
}

impl OverlookApp {
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("View", |ui| {
                    if ui.selectable_label(self.current_view == ViewMode::Chat, "💬 Chat").clicked() {
                        self.current_view = ViewMode::Chat;
                        ui.close();
                    }
                    if ui.selectable_label(self.current_view == ViewMode::Settings, "⚙️ Settings").clicked() {
                        self.current_view = ViewMode::Settings;
                        ui.close();
                    }
                    if ui.selectable_label(self.current_view == ViewMode::Export, "📥 Export").clicked() {
                        self.current_view = ViewMode::Export;
                        ui.close();
                    }
                    if ui.selectable_label(self.current_view == ViewMode::Organize, "📁 Organize").clicked() {
                        self.current_view = ViewMode::Organize;
                        ui.close();
                    }
                });
            });
        });
    }


    fn render_api_key_section(
        &mut self,
        ui: &mut egui::Ui,
        provider: ProviderKind,
        label: &str,
        hint: &str,
    ) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(label);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let key_ref = self.api_keys.get_mut(provider);
                    // Don't mask URLs for local providers
                    let should_mask = matches!(provider, ProviderKind::OpenRouter | ProviderKind::OpenAI | ProviderKind::Anthropic);

                    // Calculate available width for the text input
                    let available_width = ui.available_width();
                    let button_width = if should_mask { 60.0 } else { 0.0 };
                    let desired_width = (available_width - button_width - 8.0).min(600.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(key_ref)
                            .hint_text(hint)
                            .desired_width(desired_width)
                            .password(should_mask && self.api_key_masked),
                    );

                    if should_mask {
                        let toggle_label = if self.api_key_masked { "Show" } else { "Hide" };
                        if ui.button(toggle_label).clicked() {
                            self.api_key_masked = !self.api_key_masked;
                        }
                    }

                    if response.changed() {
                        self.handle_api_key_change(provider);
                    }
                });
            });
        });
    }

    fn handle_api_key_change(&mut self, provider: ProviderKind) {
        let trimmed = self.api_keys.get(provider).trim().to_string();
        *self.api_keys.get_mut(provider) = trimmed;

        // Only save API keys, not all data (chats, projects, etc.)
        if let Err(e) = crate::storage::Storage::save_api_keys(&self.api_keys) {
            println!("❌ Failed to save API keys: {}", e);
        }

        // Don't load models on every keystroke - user can use refresh button
        // This prevents lag when typing URLs for local providers
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("OpenRouter Chat");
                    ui.separator();

                    ui.label("Configure provider API keys from View → Settings");
                    ui.separator();

                    // New Chat button
                    if ui.button("+ New Chat").clicked() {
                        self.create_new_chat();
                    }

                    ui.separator();

                    // Projects section
                    self.render_projects_section(ui);

                    ui.separator();

                    // Chats list
                    self.render_chats_list(ui);
                });
            });
    }

    fn render_projects_section(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Projects").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // "All Chats" option
            let is_all_selected = self.current_project_id.is_none();
            let button = egui::Button::new("📋 All Chats").fill(if is_all_selected {
                self.current_theme.accent
            } else {
                egui::Color32::TRANSPARENT
            });
            if ui.add_sized([150.0, 25.0], button).clicked() {
                self.current_project_id = None;
            }

            // Add project button
            if ui.button("➕").clicked() {
                self.new_project_name.clear();
                self.project_modal_mode = crate::app::ProjectModalMode::Create;
                self.show_project_modal = true;
            }
        });

        ui.add_space(4.0);

        // List of projects
        let mut projects_vec: Vec<_> = self.projects.values().cloned().collect();
        projects_vec.sort_by(|a, b| a.name.cmp(&b.name));

        let mut project_to_select: Option<String> = None;
        let mut project_to_rename: Option<String> = None;
        let mut project_to_delete: Option<String> = None;

        for project in projects_vec {
            let is_selected = self.current_project_id.as_ref() == Some(&project.id);

            ui.horizontal(|ui| {
                let button = egui::Button::new(format!("📁 {}", project.name)).fill(if is_selected {
                    self.current_theme.accent
                } else {
                    egui::Color32::TRANSPARENT
                });

                if ui.add_sized([150.0, 25.0], button).clicked() {
                    project_to_select = Some(project.id.clone());
                }

                // Context menu for rename/delete
                if ui.small_button("⋯").clicked() {
                    // For now, we'll handle this with separate buttons
                }

                if ui.small_button("✏").clicked() {
                    project_to_rename = Some(project.id.clone());
                }

                if ui.small_button("🗑").clicked() {
                    project_to_delete = Some(project.id.clone());
                }
            });
        }

        // Handle actions
        if let Some(project_id) = project_to_select {
            self.current_project_id = Some(project_id);
        }

        if let Some(project_id) = project_to_rename {
            if let Some(project) = self.projects.get(&project_id) {
                self.new_project_name = project.name.clone();
                self.project_modal_mode = crate::app::ProjectModalMode::Rename;
                self.project_modal_target_id = Some(project_id);
                self.show_project_modal = true;
            }
        }

        if let Some(project_id) = project_to_delete {
            self.delete_project(&project_id);
        }
    }

    fn render_chats_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let chats_vec = self.get_filtered_chats();

            let mut chat_to_delete: Option<String> = None;
            let mut chat_to_select: Option<String> = None;

            for chat in chats_vec.iter() {
                let is_selected = self.current_chat_id.as_ref() == Some(&chat.id);

                ui.horizontal(|ui| {
                    let available_width = ui.available_width();
                    let button_area_width = 40.0; // Width for trash button area
                    let title_width = available_width - button_area_width;

                    // Chat title button (left-aligned, takes most of the width)
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(title_width, 30.0),
                        egui::Sense::click()
                    );

                    if is_selected {
                        ui.painter().rect_filled(
                            rect,
                            4.0,
                            self.current_theme.accent,
                        );
                    }

                    let text_pos = rect.left_center() + egui::vec2(8.0, 0.0);
                    ui.painter().text(
                        text_pos,
                        egui::Align2::LEFT_CENTER,
                        &chat.title,
                        egui::FontId::proportional(14.0),
                        if is_selected {
                            self.current_theme.button_primary_text
                        } else {
                            ui.style().visuals.text_color()
                        },
                    );

                    if response.clicked() {
                        chat_to_select = Some(chat.id.clone());
                    }

                    // Reserve space for button area and draw opaque background
                    let cursor_before = ui.cursor().min;

                    // Draw opaque background to cover text overflow
                    let button_bg_rect = egui::Rect::from_min_size(
                        cursor_before,
                        egui::vec2(button_area_width, 30.0)
                    );
                    ui.painter().rect_filled(
                        button_bg_rect,
                        0.0,
                        ui.style().visuals.window_fill(),
                    );

                    // Draw trash button
                    if ui.small_button("🗑").clicked() {
                        chat_to_delete = Some(chat.id.clone());
                    }
                });
            }

            // Handle actions outside the loop
            if let Some(chat_id) = chat_to_select {
                self.current_chat_id = Some(chat_id.clone());
                if let Some((provider, model)) = self
                    .chats
                    .get(&chat_id)
                    .map(|chat| (chat.provider, chat.model.clone()))
                {
                    self.set_current_provider(provider);
                    self.current_model = model;
                    self.refresh_token_usage_for_current_chat();
                }
            }

            if let Some(chat_id) = chat_to_delete {
                self.delete_chat(&chat_id);
            }
        });
    }

    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                ViewMode::Chat => {
                    ui.vertical(|ui| {
                        self.render_header(ui);
                        self.render_token_usage(ui);
                        ui.separator();
                        self.render_chat_area(ui);
                        ui.separator();
                        self.render_input_area(ui, ctx);
                    });
                }
                ViewMode::Settings => {
                    self.render_settings_view(ui);
                }
                ViewMode::Export => {
                    self.render_export_view(ui);
                }
                ViewMode::Organize => {
                    self.render_organize_view(ui);
                }
            }
        });
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Provider:");

            let mut selected_provider = self.current_provider;
            egui::ComboBox::from_id_salt("provider_selector")
                .selected_text(self.current_provider.display_name())
                .show_ui(ui, |ui| {
                    for provider in ProviderKind::all() {
                        ui.selectable_value(
                            &mut selected_provider,
                            provider,
                            provider.display_name(),
                        );
                    }
                });

            if selected_provider != self.current_provider {
                self.set_current_provider(selected_provider);
                self.reset_token_usage();
            }

            ui.add_space(12.0);
            ui.label("Model:");

            let provider_has_key = !self.api_keys.get(self.current_provider).trim().is_empty();

            let combo_label = if self.loading_models {
                "Loading models...".to_string()
            } else if !provider_has_key {
                "Enter API key first".to_string()
            } else if self.available_models.is_empty() {
                "No models loaded".to_string()
            } else if self.current_model.is_empty() {
                "Select a model...".to_string()
            } else {
                self.available_models
                    .iter()
                    .find(|m| m.id == self.current_model)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| self.current_model.clone())
            };

            // Model picker button (opens modal)
            ui.add_enabled_ui(provider_has_key, |ui| {
                if ui.button(format!("🔍 {}", combo_label)).clicked() {
                    self.show_model_picker_modal = true;
                    self.model_search_query.clear();
                }
            });

            if !provider_has_key {
                ui.label(
                    egui::RichText::new("← add a key in Settings")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
            }

            if ui
                .add_enabled(
                    !self.loading_models && provider_has_key,
                    egui::Button::new("🔄"),
                )
                .clicked()
            {
                self.load_models();
            }
        });
    }

    fn render_token_usage(&self, ui: &mut egui::Ui) {
        let has_usage = self.token_usage.prompt_tokens > 0
            || self.token_usage.completion_tokens > 0
            || self.token_usage.max_context_tokens.is_some();

        if !has_usage {
            return;
        }

        ui.add_space(6.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("🔼 Prompt: {}", self.token_usage.prompt_tokens));
                ui.separator();
                ui.label(format!(
                    "🔽 Completion: {}",
                    self.token_usage.completion_tokens
                ));
                ui.separator();
                if let Some(max) = self.token_usage.max_context_tokens {
                    ui.label(format!(
                        "Total: {} / {} tokens",
                        self.token_usage.total_tokens, max
                    ));

                    if let Some(percent) = self.token_usage.percentage_of_max() {
                        ui.separator();
                        ui.label(format!("{:.1}% used", percent));
                    }

                    if let Some(remaining) = self.token_usage.remaining_tokens() {
                        ui.separator();
                        ui.label(format!("Remaining: {}", remaining));
                    }
                } else {
                    ui.label(format!("Total: {} tokens", self.token_usage.total_tokens));
                }
            });
        });
        ui.add_space(6.0);
    }

    fn render_chat_area(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height() - 120.0;
        egui::ScrollArea::vertical()
            .max_height(available_height)
            .stick_to_bottom(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(chat_id) = &self.current_chat_id {
                    if let Some(chat) = self.chats.get(chat_id) {
                        let hide_streaming_assistant = self.streaming_active
                            && self.streaming_message_persisted
                            && self
                                .streaming_chat_id
                                .as_ref()
                                .map(|id| id == chat_id)
                                .unwrap_or(false);

                        let visible_count = if hide_streaming_assistant {
                            std::cmp::min(self.streaming_message_base_len, chat.messages.len())
                        } else {
                            chat.messages.len()
                        };

                        // Clone messages to avoid borrow checker issues
                        let messages_to_render: Vec<_> = chat.messages.iter().take(visible_count).cloned().collect();

                        for message in messages_to_render {
                            self.render_message(ui, &message);
                        }

                        // Show streaming message during both active streaming and character display phases
                        if self.streaming_active && !self.streaming_message.is_empty() {
                            self.render_streaming_message(ui);
                        }

                        // Show status messages
                        if self.streaming_active && !self.stream_completed {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(
                                    egui::RichText::new("🔄 AI is streaming...")
                                        .color(egui::Color32::GRAY)
                                        .italics(),
                                );
                            });
                        } else if self.sending_message {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(
                                    egui::RichText::new("AI is typing...")
                                        .color(egui::Color32::GRAY)
                                        .italics(),
                                );
                            });
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("Welcome to Overlook Chat");
                        ui.add_space(10.0);
                        ui.label(
                            "Select a model and start chatting, or create a new conversation.",
                        );
                    });
                }
            });
    }

    fn render_message(&mut self, ui: &mut egui::Ui, message: &crate::types::Message) {
        ui.add_space(5.0);

        // Handle search results specially with collapsible UI
        if message.role == "system" && message.content.starts_with("# Web Search Results") {
            self.render_search_results(ui, message);
            return;
        }

        if message.role == "user" {
            // User message - right aligned with blue background
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.add_space(20.0); // Right margin

                ui.allocate_ui_with_layout(
                    [ui.available_width() * 0.7, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.style_mut().visuals.override_text_color = Some(self.current_theme.user_message_text);
                        ui.visuals_mut().extreme_bg_color = self.current_theme.user_message_bg;

                        // Suppress egui grid ID warnings in debug mode
                        ui.ctx().options_mut(|o| o.warn_on_id_clash = false);

                        egui_commonmark::CommonMarkViewer::new()
                            .show(ui, &mut self.markdown_cache, &message.content);
                    },
                );
            });
        } else {
            // Assistant message - left aligned with gray background
            ui.horizontal(|ui| {
                ui.add_space(20.0); // Left margin

                ui.allocate_ui_with_layout(
                    [ui.available_width() * 0.8, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        // Show model indicator badge if available
                        if let (Some(provider), Some(model)) = (&message.provider, &message.model) {
                            ui.label(
                                egui::RichText::new(format!("{}: {}", provider.display_name(), model))
                                    .size(self.font_size * 0.75)
                                    .color(egui::Color32::GRAY)
                                    .italics(),
                            );
                            ui.add_space(2.0);
                        }

                        ui.style_mut().visuals.override_text_color = Some(self.current_theme.assistant_message_text);
                        ui.visuals_mut().extreme_bg_color = self.current_theme.assistant_message_bg;

                        // Suppress egui grid ID warnings in debug mode
                        ui.ctx().options_mut(|o| o.warn_on_id_clash = false);

                        egui_commonmark::CommonMarkViewer::new()
                            .show(ui, &mut self.markdown_cache, &message.content);
                    },
                );
            });
        }
        ui.add_space(8.0);
    }

    fn render_search_results(&mut self, ui: &mut egui::Ui, message: &crate::types::Message) {
        // Create a unique ID for this message using timestamp
        let message_id = format!("{}", message.timestamp.timestamp_millis());

        // Count results by counting "## Result" occurrences
        let result_count = message.content.matches("## Result").count();

        // Check if expanded
        let is_expanded = self.expanded_search_results.contains(&message_id);

        ui.horizontal(|ui| {
            ui.add_space(20.0); // Left margin

            ui.allocate_ui_with_layout(
                [ui.available_width() * 0.9, 0.0].into(),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    // Collapsible header with proper theme colors
                    let header_text = if is_expanded {
                        format!("🔍 Web Search Results ({} results) ▼ Hide", result_count)
                    } else {
                        format!("🔍 Web Search Results ({} results) ▶ Show", result_count)
                    };

                    let header_response = ui.add(
                        egui::Button::new(
                            egui::RichText::new(header_text)
                                .color(self.current_theme.assistant_message_text)
                        )
                        .fill(self.current_theme.assistant_message_bg)
                        .min_size(egui::vec2(ui.available_width(), 30.0))
                    );

                    if header_response.clicked() {
                        if is_expanded {
                            self.expanded_search_results.remove(&message_id);
                        } else {
                            self.expanded_search_results.insert(message_id.clone());
                        }
                    }

                    // Show full content if expanded
                    if is_expanded {
                        ui.add_space(8.0);

                        ui.style_mut().visuals.override_text_color = Some(self.current_theme.assistant_message_text);
                        ui.visuals_mut().extreme_bg_color = self.current_theme.assistant_message_bg;

                        // Suppress egui grid ID warnings
                        ui.ctx().options_mut(|o| o.warn_on_id_clash = false);

                        egui_commonmark::CommonMarkViewer::new()
                            .show(ui, &mut self.markdown_cache, &message.content);
                    }
                },
            );
        });

        ui.add_space(8.0);
    }

    fn render_streaming_message(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);

        // Assistant streaming message - left aligned with gray background and typing indicator
        ui.horizontal(|ui| {
            ui.add_space(20.0); // Left margin

            ui.allocate_ui_with_layout(
                [ui.available_width() * 0.8, 0.0].into(),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    // Show model indicator badge for streaming message
                    if let (Some(provider), Some(model)) = (&self.last_request_provider, &self.last_request_model) {
                        ui.label(
                            egui::RichText::new(format!("{}: {}", provider.display_name(), model))
                                .size(self.font_size * 0.75)
                                .color(egui::Color32::GRAY)
                                .italics(),
                        );
                        ui.add_space(2.0);
                    }

                    ui.style_mut().visuals.override_text_color = Some(self.current_theme.assistant_message_text);
                    ui.visuals_mut().extreme_bg_color = self.current_theme.assistant_message_bg;

                    // Suppress egui grid ID warnings in debug mode
                    ui.ctx().options_mut(|o| o.warn_on_id_clash = false);

                    // Render markdown content
                    egui_commonmark::CommonMarkViewer::new()
                        .show(ui, &mut self.markdown_cache, &self.streaming_message);

                    // Add a blinking cursor effect only while actively streaming
                    if !self.stream_completed {
                        let time = ui.input(|i| i.time);
                        if (time * 2.0) as i32 % 2 == 0 {
                            ui.label(
                                egui::RichText::new("▊")
                                    .color(self.current_theme.assistant_message_text)
                                    .background_color(self.current_theme.assistant_message_bg)
                            );
                        }
                    }
                },
            );
        });
        ui.add_space(8.0);
    }

    fn render_input_area(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            ui.add_space(5.0);

            // Attachment area
            if !self.selected_attachments.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Attachments:");
                    let mut to_remove = None;
                    for (i, file_path) in self.selected_attachments.iter().enumerate() {
                        let file_name = std::path::Path::new(file_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        ui.horizontal(|ui| {
                            ui.label(format!("📎 {}", file_name));
                            if ui.small_button("❌").clicked() {
                                to_remove = Some(i);
                            }
                        });
                    }

                    if let Some(i) = to_remove {
                        self.selected_attachments.remove(i);
                    }
                });
                ui.add_space(5.0);
            }

            // Input area with better sizing
            ui.horizontal(|ui| {
                // Attach button
                if ui.button("📎 Attach").clicked() {
                    self.open_file_picker();
                }

                // Calculate available width for the text input
                let available_width = ui.available_width();
                let input_width = (available_width - 80.0).min(800.0);

                // Large text input that takes most of the width
                let text_edit_response = ui.add_sized(
                    [input_width, 60.0], // Make input much taller
                    egui::TextEdit::multiline(&mut self.message_input)
                        .hint_text(
                            "Type your message here... (Shift+Enter for new line, Enter to send)",
                        )
                        .desired_rows(3),
                );

                // Handle Enter key to send (but not Shift+Enter)
                if text_edit_response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                    && !self.sending_message
                    && (!self.streaming_active || self.stream_completed)
                {
                    self.send_message(ctx);
                }

                // Send button
                ui.vertical(|ui| {
                    ui.add_space(15.0); // Center the button vertically
                    let button_text = if self.streaming_active && !self.stream_completed {
                        "Streaming..."
                    } else if self.sending_message {
                        "Sending..."
                    } else {
                        "Send"
                    };

                    let send_btn = ui.add_sized([70.0, 30.0], egui::Button::new(button_text));

                    if send_btn.clicked()
                        && !self.sending_message
                        && (!self.streaming_active || self.stream_completed)
                    {
                        self.send_message(ctx);
                    }
                });
            });

            ui.add_space(5.0);
        });
    }

    fn render_modals(&mut self, ctx: &egui::Context) {
        // Project modal (keep this as it's still used in the sidebar)
        if self.show_project_modal {
            let title = match self.project_modal_mode {
                crate::app::ProjectModalMode::Create => "New Project",
                crate::app::ProjectModalMode::Rename => "Rename Project",
            };

            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Project name:");
                    let response = ui.text_edit_singleline(&mut self.new_project_name);

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Submit on Enter
                        match self.project_modal_mode {
                            crate::app::ProjectModalMode::Create => {
                                self.create_project(self.new_project_name.clone());
                            }
                            crate::app::ProjectModalMode::Rename => {
                                if let Some(project_id) = self.project_modal_target_id.clone() {
                                    self.rename_project(project_id, self.new_project_name.clone());
                                }
                            }
                        }
                        self.show_project_modal = false;
                        self.new_project_name.clear();
                    }

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        let button_text = match self.project_modal_mode {
                            crate::app::ProjectModalMode::Create => "Create",
                            crate::app::ProjectModalMode::Rename => "Rename",
                        };

                        if ui.button(button_text).clicked() && !self.new_project_name.trim().is_empty() {
                            match self.project_modal_mode {
                                crate::app::ProjectModalMode::Create => {
                                    self.create_project(self.new_project_name.clone());
                                }
                                crate::app::ProjectModalMode::Rename => {
                                    if let Some(project_id) = self.project_modal_target_id.clone() {
                                        self.rename_project(project_id, self.new_project_name.clone());
                                    }
                                }
                            }
                            self.show_project_modal = false;
                            self.new_project_name.clear();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_project_modal = false;
                            self.new_project_name.clear();
                        }
                    });
                });
        }

        // Model picker modal
        if self.show_model_picker_modal {
            egui::Window::new("Select Model")
                .collapsible(false)
                .resizable(true)
                .default_width(700.0)
                .default_height(600.0)
                .show(ctx, |ui| {
                    self.render_model_picker_content(ui);
                });
        }
    }

    fn render_model_picker_content(&mut self, ui: &mut egui::Ui) {
        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            let response = ui.add_sized(
                [ui.available_width() - 80.0, 20.0],
                egui::TextEdit::singleline(&mut self.model_search_query).hint_text("Type to filter models...")
            );

            if response.changed() {
                // Search query changed - UI will update automatically
            }

            if ui.button("Clear").clicked() {
                self.model_search_query.clear();
            }
        });

        ui.add_space(8.0);

        // Get filtered and grouped models
        let grouped_models = self.get_filtered_grouped_models();
        let total_filtered = grouped_models.iter().map(|(_, models)| models.len()).sum::<usize>();

        // Show count
        ui.label(format!(
            "Showing {} of {} models",
            total_filtered,
            self.available_models.len()
        ));

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Favorites section (if any)
        if !self.favorite_models.is_empty() {
            ui.collapsing("⭐ Favorites", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        let favorites: Vec<_> = self.available_models
                            .iter()
                            .filter(|m| self.favorite_models.contains(&m.id))
                            .cloned()
                            .collect();

                        for model in favorites {
                            self.render_model_item(ui, &model, true);
                        }
                    });
            });
            ui.add_space(8.0);
        }

        // Grouped models list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (category, models) in grouped_models {
                    ui.collapsing(format!("{} ({})", category, models.len()), |ui| {
                        for model in &models {
                            self.render_model_item(ui, model, false);
                        }
                    });
                }
            });

        ui.add_space(12.0);
        ui.separator();

        // Bottom buttons
        ui.horizontal(|ui| {
            if ui.button("Close").clicked() {
                self.show_model_picker_modal = false;
                self.model_search_query.clear();
            }
        });
    }

    fn render_model_item(&mut self, ui: &mut egui::Ui, model: &crate::types::ModelDescriptor, _in_favorites: bool) {
        ui.horizontal(|ui| {
            // Favorite toggle
            let is_favorite = self.favorite_models.contains(&model.id);
            let star_icon = if is_favorite { "⭐" } else { "☆" };

            if ui.small_button(star_icon).clicked() {
                if is_favorite {
                    self.favorite_models.retain(|id| id != &model.id);
                } else {
                    self.favorite_models.push(model.id.clone());
                }
                // Save favorites immediately
                if let Err(e) = crate::storage::Storage::save_favorite_models(&self.favorite_models) {
                    println!("❌ Failed to save favorite models: {}", e);
                }
            }

            // Model name as selectable label (left-aligned)
            let is_selected = self.current_model == model.id;
            let available_width = ui.available_width() - 150.0;

            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(available_width, 24.0),
                egui::Sense::click()
            );

            if is_selected {
                ui.painter().rect_filled(
                    rect,
                    4.0,
                    self.current_theme.accent,
                );
            }

            let text_pos = rect.left_center() + egui::vec2(8.0, 0.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                &model.name,
                egui::FontId::proportional(14.0),
                if is_selected {
                    self.current_theme.button_primary_text
                } else {
                    ui.style().visuals.text_color()
                },
            );

            if response.clicked() {
                self.current_model = model.id.clone();
                self.show_model_picker_modal = false;
                self.model_search_query.clear();
                self.on_model_changed();
            }

            // Context length badge
            if let Some(context_len) = model.context_length {
                ui.label(
                    egui::RichText::new(format!("{}k", context_len / 1000))
                        .size(11.0)
                        .color(egui::Color32::GRAY)
                );
            }
        })
        .response
        .on_hover_text_at_pointer(
            model.description.as_ref()
                .map(|d| d.as_str())
                .unwrap_or("No description available")
        );
    }

    fn render_settings_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // API Keys Section
                ui.group(|ui| {
                    ui.heading("Provider API Keys");
                    ui.add_space(6.0);
                    ui.label("Keys are stored locally on this device.");
                    ui.add_space(12.0);

                    self.render_api_key_section(
                        ui,
                        ProviderKind::OpenRouter,
                        "OpenRouter",
                        "Enter OpenRouter API key",
                    );

                    ui.add_space(12.0);

                    self.render_api_key_section(ui, ProviderKind::OpenAI, "OpenAI", "Enter OpenAI API key");

                    ui.add_space(12.0);

                    self.render_api_key_section(
                        ui,
                        ProviderKind::Anthropic,
                        "Anthropic",
                        "Enter Anthropic API key",
                    );

                    ui.add_space(12.0);

                    self.render_api_key_section(
                        ui,
                        ProviderKind::LMStudio,
                        "LM Studio",
                        "Enter LM Studio URL (e.g., http://localhost:1234)",
                    );

                    ui.add_space(12.0);

                    self.render_api_key_section(
                        ui,
                        ProviderKind::Ollama,
                        "Ollama",
                        "Enter Ollama URL (e.g., http://localhost:11434)",
                    );

                    ui.add_space(12.0);
                    ui.label("Updates sync immediately once you edit a key.");
                });

                ui.add_space(20.0);

                // Appearance Section
                ui.group(|ui| {
                    ui.heading("Appearance");
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        let before_theme = self.current_theme_type;
                        egui::ComboBox::from_id_salt("settings_theme_selector")
                            .selected_text(self.current_theme_type.name())
                            .show_ui(ui, |ui| {
                                for theme_type in ThemeType::all_themes() {
                                    ui.selectable_value(
                                        &mut self.current_theme_type,
                                        theme_type,
                                        theme_type.name(),
                                    );
                                }
                            });

                        if self.current_theme_type != before_theme {
                            self.current_theme = Theme::from_theme_type(&self.current_theme_type);
                            ui.ctx().set_style(egui::Style::default());
                            ui.ctx().style_mut(|style| {
                                self.current_theme.apply_to_egui_style(style, self.font_size);
                            });
                            self.save_data();
                            ui.ctx().request_repaint();
                        }
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        let before_font_size = self.font_size;
                        ui.add(egui::Slider::new(&mut self.font_size, 10.0..=20.0).suffix("pt"));

                        if (self.font_size - before_font_size).abs() > 0.1 {
                            ui.ctx().set_style(egui::Style::default());
                            ui.ctx().style_mut(|style| {
                                self.current_theme.apply_to_egui_style(style, self.font_size);
                            });
                            self.save_data();
                            ui.ctx().request_repaint();
                        }
                    });
                });

                ui.add_space(20.0);

                // Streaming Section
                ui.group(|ui| {
                    ui.heading("Chat Behavior");
                    ui.add_space(12.0);
                    ui.checkbox(&mut self.streaming_enabled, "Enable streaming responses");
                });
            });
    }

    fn render_export_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Export");
        ui.add_space(12.0);

        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            if self.current_chat_id.is_some() {
                ui.label("Export the current chat conversation to a file.");
                ui.add_space(20.0);

                if ui.button("📥 Export Current Chat").clicked() {
                    self.export_current_chat();
                }

                // Show export result
                if let Some(result) = &self.last_export_result {
                    ui.add_space(20.0);
                    match result {
                        Ok(path) => {
                            ui.label(
                                egui::RichText::new("✅ Export successful!")
                                    .color(egui::Color32::GREEN)
                                    .strong(),
                            );
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Saved to:").italics());
                            ui.label(egui::RichText::new(path).monospace());
                        }
                        Err(err) => {
                            ui.label(
                                egui::RichText::new(format!("❌ Export failed: {}", err))
                                    .color(egui::Color32::RED),
                            );
                        }
                    }
                }
            } else {
                ui.label("Select a chat to export.");
            }
        });
    }

    fn render_organize_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Organize");
        ui.add_space(12.0);

        if let Some(chat_id) = &self.current_chat_id.clone() {
            ui.label("Move the current chat to a project:");
            ui.add_space(20.0);

            if ui.button("📋 No Project (Unassigned)").clicked() {
                self.move_chat_to_project(chat_id, None);
            }

            ui.add_space(10.0);

            let mut projects_vec: Vec<_> = self.projects.values().cloned().collect();
            projects_vec.sort_by(|a, b| a.name.cmp(&b.name));

            for project in projects_vec {
                if ui.button(format!("📁 {}", project.name)).clicked() {
                    self.move_chat_to_project(chat_id, Some(project.id.clone()));
                }
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Select a chat to organize.");
            });
        }
    }
}
