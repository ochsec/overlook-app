use crate::app::OverlookApp;
use crate::theme::{Theme, ThemeType};
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

        self.render_sidebar(ctx);
        self.render_main_content(ctx);
        self.render_modals(ctx);

        // Request repaint only when we actually have streaming updates
        // (The handle_streaming_tokens method will request repaints when tokens arrive)
    }
}

impl OverlookApp {
    fn render_sidebar(&mut self, ctx: &egui::Context) {
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
                self.render_chats_list(ui);
            });
        });
    }

    fn render_chats_list(&mut self, ui: &mut egui::Ui) {
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
                if let Some(current_chat) = self.chats.get(&chat_id) {
                    self.current_model = current_chat.model.clone();
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
                ui.separator();
                self.render_chat_area(ui);
                ui.separator();
                self.render_input_area(ui, ctx);
            });
        });
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
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

                // Theme selector
                ui.separator();
                ui.label("Theme:");
                let before_theme = self.current_theme_type;
                egui::ComboBox::from_id_source("theme_selector")
                    .selected_text(self.current_theme_type.name())
                    .show_ui(ui, |ui| {
                        for theme_type in ThemeType::all_themes() {
                            ui.selectable_value(&mut self.current_theme_type, theme_type, theme_type.name());
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
                        println!("Theme changed from {:?} to {:?}", before_theme, self.current_theme_type);
                        self.current_theme = Theme::from_theme_type(&self.current_theme_type);
                    }
                    if font_changed {
                        println!("Font size changed to: {:.1}pt", self.font_size);
                    }

                    // Force complete style refresh by setting to default first, then applying theme
                    ui.ctx().set_style(egui::Style::default());
                    ui.ctx().style_mut(|style| {
                        self.current_theme.apply_to_egui_style(style, self.font_size);
                    });

                    self.save_data();

                    // Request repaint to ensure changes are visible immediately
                    ui.ctx().request_repaint();
                }
            });
        });
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
                        for message in &chat.messages {
                            self.render_message(ui, message);
                        }

                        // Show streaming message if active
                        if self.streaming_active && !self.streaming_message.is_empty() {
                            self.render_streaming_message(ui);
                        }

                        // Show status messages
                        if self.streaming_active && !self.stream_completed {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("🔄 AI is streaming...")
                                    .color(egui::Color32::GRAY)
                                    .italics());
                            });
                        } else if self.sending_message {
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
                        color: self.current_theme.assistant_message_text,
                        background: self.current_theme.assistant_message_bg,
                        font_id: egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
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

    fn render_streaming_message(&self, ui: &mut egui::Ui) {
        ui.add_space(5.0);

        // Assistant streaming message - left aligned with gray background and typing indicator
        ui.horizontal(|ui| {
            ui.add_space(20.0); // Left margin

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
                            font_id: egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
                            ..Default::default()
                        },
                    );
                }
            }

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
                        .hint_text("Type your message here... (Shift+Enter for new line, Enter to send)")
                        .desired_rows(3)
                );

                // Handle Enter key to send (but not Shift+Enter)
                if text_edit_response.has_focus() && ui.input(|i| {
                    i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                }) && !self.sending_message && (!self.streaming_active || self.stream_completed) {
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

                    let send_btn = ui.add_sized(
                        [70.0, 30.0],
                        egui::Button::new(button_text)
                    );

                    if send_btn.clicked() && !self.sending_message && (!self.streaming_active || self.stream_completed) {
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
    }
}