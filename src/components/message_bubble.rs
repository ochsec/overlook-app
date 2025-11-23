// Message Bubble Component - Renders chat messages
use gpui::*;
use gpui::prelude::*;
use crate::gpui_theme::GpuiTheme;
use crate::types::Message;

pub struct MessageBubble {
    message: Message,
    theme: GpuiTheme,
}

impl MessageBubble {
    pub fn new(message: Message, theme: GpuiTheme) -> Self {
        Self { message, theme }
    }

    pub fn render_element(self) -> impl IntoElement {
        let is_user = self.message.role == "user";
        let is_system = self.message.role == "system";

        if is_system {
            // System messages (like search results) - centered, muted
            div()
                .flex()
                .justify_center()
                .py_2()
                .child(
                    div()
                        .max_w(px(800.0))
                        .px_4()
                        .py_2()
                        .bg(self.theme.assistant_message_bg)
                        .rounded_lg()
                        .text_color(self.theme.text_muted)
                        .text_sm()
                        .italic()
                        .child(self.message.content.clone())
                )
        } else if is_user {
            // User message - right aligned, accent color
            div()
                .flex()
                .justify_end()
                .py_2()
                .pr_4()
                .child(
                    div()
                        .max_w_3_4() // 75% max width
                        .px_4()
                        .py_3()
                        .bg(self.theme.user_message_bg)
                        .rounded_lg()
                        .text_color(self.theme.user_message_text)
                        .child(self.message.content.clone())
                )
        } else {
            // Assistant message - left aligned, surface color
            div()
                .flex()
                .flex_col()
                .py_2()
                .pl_4()
                .gap_1()
                // Model badge (if available)
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
                            .text_xs()
                            .text_color(self.theme.text_muted)
                            .italic()
                            .child(format!("{}: {}", provider_name, model_name))
                    )
                })
                .child(
                    div()
                        .max_w_4_5() // 80% max width
                        .px_4()
                        .py_3()
                        .bg(self.theme.assistant_message_bg)
                        .rounded_lg()
                        .text_color(self.theme.assistant_message_text)
                        // TODO: Add markdown rendering
                        .child(self.message.content.clone())
                )
        }
    }
}

// Helper to render a streaming message with cursor
pub fn render_streaming_message(content: &str, theme: &GpuiTheme, show_cursor: bool) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .py_2()
        .pl_4()
        .gap_1()
        .child(
            div()
                .max_w_4_5()
                .px_4()
                .py_3()
                .bg(theme.assistant_message_bg)
                .rounded_lg()
                .text_color(theme.assistant_message_text)
                .child(
                    div()
                        .flex()
                        .child(content.to_string())
                        .when(show_cursor, |this| {
                            this.child(
                                div()
                                    .text_color(theme.assistant_message_text)
                                    .child("▊")
                            )
                        })
                )
        )
}
