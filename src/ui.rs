use crate::app::OverlookApp;
use crate::theme::{Theme, ThemeType};
use crate::types::ProviderKind;
use eframe::egui::{self, ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};

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

        self.render_menu_bar(ctx);
        self.render_api_settings_window(ctx);
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
                ui.menu_button("Settings", |ui| {
                    if ui.button("API Keys…").clicked() {
                        self.show_api_settings_window = true;
                        ui.close();
                    }
                });
            });
        });
    }

    fn render_api_settings_window(&mut self, ctx: &egui::Context) {
        let viewport_id = ViewportId::from_hash_of("api_settings_window");

        if !self.show_api_settings_window {
            ctx.send_viewport_cmd_to(viewport_id, ViewportCommand::Close);
            return;
        }

        let builder = ViewportBuilder::default()
            .with_title("API Keys")
            .with_inner_size([450.0, 400.0])
            .with_min_inner_size([400.0, 250.0])
            .with_max_inner_size([600.0, 500.0])
            .with_resizable(true);

        ctx.show_viewport_immediate(viewport_id, builder, |api_ctx, class| match class {
            ViewportClass::Immediate | ViewportClass::Deferred | ViewportClass::Root => {
                self.render_api_settings_panel(api_ctx);

                if api_ctx.input(|i| i.viewport().close_requested()) {
                    self.show_api_settings_window = false;
                }
            }
            ViewportClass::Embedded => {
                let mut open = true;
                let mut should_close = false;

                egui::Window::new("API Keys")
                    .open(&mut open)
                    .resizable(true)
                    .collapsible(false)
                    .show(api_ctx, |ui| {
                        should_close = self.render_api_settings_ui(ui);
                    });

                if api_ctx.input(|i| i.viewport().close_requested()) || !open || should_close {
                    self.show_api_settings_window = false;
                }
            }
        });
    }

    fn render_api_settings_panel(&mut self, ctx: &egui::Context) {
        let mut should_close = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            should_close = self.render_api_settings_ui(ui);
        });

        if should_close {
            self.show_api_settings_window = false;
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn render_api_settings_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut should_close = false;

        ui.heading("Provider API Keys");
        ui.add_space(6.0);
        ui.label("Keys are stored locally on this device.");
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
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

        ui.add_space(12.0);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Close").clicked() {
                should_close = true;
            }
        });

        should_close
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
                    let response = ui.add(
                        egui::TextEdit::singleline(key_ref)
                            .hint_text(hint)
                            .desired_width(f32::INFINITY)
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

        self.save_data();

        if provider == self.current_provider {
            self.load_models();
        }
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("OpenRouter Chat");
                    ui.separator();

                    ui.label("Configure provider API keys from Settings → API Keys…");
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
                    let button = egui::Button::new(&chat.title).fill(if is_selected {
                        self.current_theme.accent
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
            ui.vertical(|ui| {
                self.render_header(ui);
                self.render_token_usage(ui);
                ui.separator();
                self.render_chat_area(ui);
                ui.separator();
                self.render_input_area(ui, ctx);
            });
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

            let previous_model = self.current_model.clone();
            ui.add_enabled_ui(provider_has_key, |ui| {
                egui::ComboBox::from_id_salt("model_selector")
                    .selected_text(combo_label.clone())
                    .show_ui(ui, |ui| {
                        for model in &self.available_models {
                            ui.selectable_value(
                                &mut self.current_model,
                                model.id.clone(),
                                &model.name,
                            );
                        }
                    });
            });

            if self.current_model != previous_model {
                self.on_model_changed();
            }

            if !provider_has_key {
                ui.label(
                    egui::RichText::new("← add a key")
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

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📥 Export").clicked() {
                    self.show_export_modal = true;
                }

                if ui.button("📁 Organize").clicked() {
                    self.show_organize_modal = true;
                }

                // Theme selector
                ui.separator();
                ui.label("Theme:");
                let before_theme = self.current_theme_type;
                egui::ComboBox::from_id_salt("theme_selector")
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

                // Font size control
                ui.separator();
                ui.label("Font Size:");
                let before_font_size = self.font_size;
                ui.add(egui::Slider::new(&mut self.font_size, 10.0..=20.0).suffix("pt"));

                // Streaming toggle
                ui.separator();
                ui.checkbox(&mut self.streaming_enabled, "🔄 Stream");

                // Check if theme or font size changed
                let theme_changed = self.current_theme_type != before_theme;
                let font_changed = (self.font_size - before_font_size).abs() > 0.1;

                if theme_changed || font_changed {
                    if theme_changed {
                        println!(
                            "Theme changed from {:?} to {:?}",
                            before_theme, self.current_theme_type
                        );
                        self.current_theme = Theme::from_theme_type(&self.current_theme_type);
                    }
                    if font_changed {
                        println!("Font size changed to: {:.1}pt", self.font_size);
                    }

                    // Force complete style refresh by setting to default first, then applying theme
                    ui.ctx().set_style(egui::Style::default());
                    ui.ctx().style_mut(|style| {
                        self.current_theme
                            .apply_to_egui_style(style, self.font_size);
                    });

                    self.save_data();

                    // Request repaint to ensure changes are visible immediately
                    ui.ctx().request_repaint();
                }
            });
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

                        for message in chat.messages.iter().take(visible_count) {
                            self.render_message(ui, message);
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

    fn render_message(&self, ui: &mut egui::Ui, message: &crate::types::Message) {
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
                        color: self.current_theme.user_message_text,
                        background: self.current_theme.user_message_bg,
                        font_id: egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
                        ..Default::default()
                    },
                );

                ui.allocate_ui_with_layout(
                    [ui.available_width() * 0.7, 0.0].into(),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add(egui::Label::new(job).wrap().selectable(true));
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

                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            &message.content,
                            0.0,
                            egui::TextFormat {
                                color: self.current_theme.assistant_message_text,
                                background: self.current_theme.assistant_message_bg,
                                font_id: egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
                                ..Default::default()
                            },
                        );

                        ui.add(egui::Label::new(job).wrap().selectable(true));
                    },
                );
            });
        }
        ui.add_space(8.0);
    }

    fn render_streaming_message(&self, ui: &mut egui::Ui) {
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

                    let mut job = egui::text::LayoutJob::default();
                    job.append(
                        &self.streaming_message,
                        0.0,
                        egui::TextFormat {
                            color: self.current_theme.assistant_message_text,
                            background: self.current_theme.assistant_message_bg,
                            font_id: egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
                            ..Default::default()
                        },
                    );

                    // Add a blinking cursor effect only while actively streaming
                    if !self.stream_completed {
                        let time = ui.input(|i| i.time);
                        if (time * 2.0) as i32 % 2 == 0 {
                            job.append(
                                "▊",
                                0.0,
                                egui::TextFormat {
                                    color: self.current_theme.assistant_message_text,
                                    background: self.current_theme.assistant_message_bg,
                                    font_id: egui::FontId::new(
                                        self.font_size,
                                        egui::FontFamily::Proportional,
                                    ),
                                    ..Default::default()
                                },
                            );
                        }
                    }

                    ui.add(egui::Label::new(job).wrap().selectable(true));
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

                // Large text input that takes most of the width
                let text_edit_response = ui.add_sized(
                    [ui.available_width() - 80.0, 60.0], // Make input much taller
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

        // Project modal
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

        // Organize modal (move chat to project)
        if self.show_organize_modal && self.current_chat_id.is_some() {
            let chat_id_clone = self.current_chat_id.clone();

            egui::Window::new("Move Chat to Project")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Select project:");
                    ui.add_space(8.0);

                    let mut selected_project: Option<Option<String>> = None;
                    let mut should_close = false;

                    if ui.button("📋 No Project (Unassigned)").clicked() {
                        selected_project = Some(None);
                    }

                    let mut projects_vec: Vec<_> = self.projects.values().cloned().collect();
                    projects_vec.sort_by(|a, b| a.name.cmp(&b.name));

                    for project in projects_vec {
                        if ui.button(format!("📁 {}", project.name)).clicked() {
                            selected_project = Some(Some(project.id.clone()));
                        }
                    }

                    ui.add_space(10.0);

                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }

                    // Handle selection after UI
                    if let Some(project_id) = selected_project {
                        if let Some(chat_id) = &chat_id_clone {
                            self.move_chat_to_project(chat_id, project_id);
                        }
                        should_close = true;
                    }

                    if should_close {
                        self.show_organize_modal = false;
                    }
                });
        }
    }
}
