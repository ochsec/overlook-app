// Export View - Chat export functionality
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::Disableable;
use crate::state::AppState;
use crate::export::Exporter;

pub struct ExportView {
    state: Entity<AppState>,
}

impl ExportView {
    pub fn new(state: Entity<AppState>) -> Self {
        Self { state }
    }

    fn export_chat(&self, cx: &mut Context<Self>) {
        let state = self.state.read(cx);

        let Some(chat_id) = &state.export_selected_chat_id else {
            self.state.update(cx, |state, _| {
                state.last_export_result = Some(Err("No chat selected".to_string()));
            });
            return;
        };

        let Some(chat) = state.chats.get(chat_id) else {
            self.state.update(cx, |state, _| {
                state.last_export_result = Some(Err("Chat not found".to_string()));
            });
            return;
        };

        // Generate markdown content
        let content = Exporter::generate_chat_markdown(chat);

        // Create filename from chat title
        let safe_title: String = chat.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
            .collect();
        let filename = format!("{}.md", safe_title.trim());

        // Export to file
        let result = Exporter::export_to_file(&content, &filename);

        self.state.update(cx, |state, _| {
            state.last_export_result = Some(result);
        });
    }

    fn open_exported_file(&self, cx: &Context<Self>) {
        let state = self.state.read(cx);
        if let Some(Ok(path)) = &state.last_export_result {
            let _ = open::that(path);
        }
    }
}

impl Render for ExportView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();
        let theme = state.theme.clone();
        let font_size = state.font_size;

        // Get all chats sorted by updated_at
        let mut chats: Vec<_> = state.chats.values().cloned().collect();
        chats.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        // Get selected chat info
        let selected_chat_id = state.export_selected_chat_id.clone();
        let selected_chat = selected_chat_id.as_ref()
            .and_then(|id| state.chats.get(id));

        let selected_label = selected_chat
            .map(|c| c.title.clone())
            .unwrap_or_else(|| "Select a chat...".to_string());

        let export_result = state.last_export_result.clone();
        let has_selection = selected_chat_id.is_some();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.text_primary)
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .p_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_size(px(font_size + 6.0))
                            .font_weight(FontWeight::BOLD)
                            .child("Export Chat")
                    )
            )
            // Content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p_6()
                    .gap_6()
                    // Chat selector section
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(font_size))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Select Chat to Export")
                            )
                            .child({
                                let chats_for_dropdown = chats.clone();
                                let state_model = state_model.clone();
                                Button::new("chat_selector")
                                    .label(SharedString::from(selected_label))
                                    .dropdown_menu(move |menu, _window, _cx| {
                                        let state_model = state_model.clone();
                                        let mut menu = menu;
                                        for chat in &chats_for_dropdown {
                                            let chat_id = chat.id.clone();
                                            let chat_title = chat.title.clone();
                                            let msg_count = chat.messages.len();
                                            let label = format!("{} ({} messages)", chat_title, msg_count);

                                            menu = menu.item(
                                                PopupMenuItem::new(label)
                                                    .on_click({
                                                        let state_model = state_model.clone();
                                                        let chat_id = chat_id.clone();
                                                        move |_, _, app| {
                                                            state_model.update(app, |state, _| {
                                                                state.export_selected_chat_id = Some(chat_id.clone());
                                                                state.last_export_result = None;
                                                            });
                                                        }
                                                    })
                                            );
                                        }
                                        menu
                                    })
                            })
                    )
                    // Selected chat preview
                    .when_some(selected_chat.cloned(), |this, chat| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_4()
                                .rounded_lg()
                                .border_1()
                                .border_color(theme.border)
                                .bg(theme.surface)
                                .child(
                                    div()
                                        .text_size(px(font_size))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(chat.title.clone())
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_4()
                                        .text_size(px(font_size - 2.0))
                                        .text_color(theme.text_secondary)
                                        .child(format!("{} messages", chat.messages.len()))
                                        .child(format!("Provider: {}", chat.provider.display_name()))
                                        .child(format!("Model: {}", chat.model))
                                )
                                .child(
                                    div()
                                        .text_size(px(font_size - 2.0))
                                        .text_color(theme.text_muted)
                                        .child(format!(
                                            "Created: {} | Updated: {}",
                                            chat.created_at.format("%Y-%m-%d %H:%M"),
                                            chat.updated_at.format("%Y-%m-%d %H:%M")
                                        ))
                                )
                        )
                    })
                    // Export button
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(
                                Button::new("export_btn")
                                    .label("Export to Markdown")
                                    .primary()
                                    .disabled(!has_selection)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.export_chat(cx);
                                    }))
                            )
                    )
                    // Status display
                    .when_some(export_result.clone(), |this, result| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .p_4()
                                .rounded_lg()
                                .border_1()
                                .border_color(match &result {
                                    Ok(_) => theme.accent,
                                    Err(_) => gpui::red(),
                                })
                                .child(
                                    match result {
                                        Ok(path) => {
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_color(theme.accent)
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .child("Export successful!")
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(font_size - 2.0))
                                                        .text_color(theme.text_secondary)
                                                        .child(format!("Saved to: {}", path))
                                                )
                                                .child(
                                                    Button::new("open_file")
                                                        .label("Open File")
                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                            this.open_exported_file(cx);
                                                        }))
                                                )
                                        }
                                        Err(err) => {
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_color(gpui::red())
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .child("Export failed")
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(font_size - 2.0))
                                                        .text_color(theme.text_secondary)
                                                        .child(err)
                                                )
                                        }
                                    }
                                )
                        )
                    })
                    // Help text
                    .child(
                        div()
                            .text_size(px(font_size - 2.0))
                            .text_color(theme.text_muted)
                            .child("Exports the selected chat as a Markdown file to your Downloads folder.")
                    )
            )
    }
}
