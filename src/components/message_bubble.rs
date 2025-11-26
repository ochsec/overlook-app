// Message Component - Terminal-style chat messages
use gpui::*;
use gpui::prelude::*;
use crate::gpui_theme::GpuiTheme;
use crate::types::Message;

pub struct MessageBubble {
    message: Message,
    theme: GpuiTheme,
    font_size: f32,
}

impl MessageBubble {
    pub fn new(message: Message, theme: GpuiTheme, font_size: f32) -> Self {
        Self { message, theme, font_size }
    }

    pub fn render_element(self) -> impl IntoElement {
        let is_user = self.message.role == "user";
        let is_system = self.message.role == "system";

        let font_size = self.font_size;

        if is_system {
            // System messages - muted, with # prefix
            div()
                .flex()
                .px_4()
                .py_1()
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .text_color(self.theme.text_muted)
                                .text_size(px(font_size))
                                .child("#")
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(self.theme.text_muted)
                                .text_size(px(font_size - 1.0))
                                .italic()
                                .child(self.message.content.clone())
                        )
                )
        } else if is_user {
            // User message - with > prompt prefix
            div()
                .flex()
                .px_4()
                .py_1()
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .text_color(self.theme.accent)
                                .text_size(px(font_size))
                                .font_weight(FontWeight::BOLD)
                                .child(">")
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(self.theme.text_primary)
                                .text_size(px(font_size))
                                .child(self.message.content.clone())
                        )
                )
        } else {
            // Assistant message - no prefix, model info inline
            div()
                .flex()
                .flex_col()
                .px_4()
                .py_1()
                .gap_0()
                // Model info line (if available)
                .when(self.message.provider.is_some() && self.message.model.is_some(), |this| {
                    let provider_name = self.message.provider
                        .as_ref()
                        .map(|p| p.display_name())
                        .unwrap_or("");
                    let model_name = self.message.model
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("");

                    this.child(
                        div()
                            .text_size(px(font_size - 2.0))
                            .text_color(self.theme.text_muted)
                            .child(format!("--- {} ({}) ---", model_name, provider_name))
                    )
                })
                .child(
                    div()
                        .text_color(self.theme.text_secondary)
                        .text_size(px(font_size))
                        .child(self.message.content.clone())
                )
        }
    }
}

// Helper to render a streaming message with cursor
pub fn render_streaming_message(content: &str, theme: &GpuiTheme, font_size: f32, show_cursor: bool) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .px_4()
        .py_1()
        .child(
            div()
                .flex()
                .text_color(theme.text_secondary)
                .text_size(px(font_size))
                .child(content.to_string())
                .when(show_cursor, |this| {
                    this.child(
                        div()
                            .text_color(theme.accent)
                            .child("▊")
                    )
                })
        )
}
