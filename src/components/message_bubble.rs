// Message Component - Terminal-style chat messages with markdown support
use gpui::*;
use gpui::prelude::*;
use crate::gpui_theme::GpuiTheme;
use crate::markdown::render_markdown;
use crate::types::Message;
use crate::components::attachment_preview::render_message_attachments;

pub struct MessageBubble {
    message: Message,
    theme: GpuiTheme,
    font_size: f32,
    index: usize,
}

impl MessageBubble {
    pub fn new(message: Message, theme: GpuiTheme, font_size: f32) -> Self {
        Self { message, theme, font_size, index: 0 }
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn render_element(self) -> impl IntoElement {
        let is_user = self.message.role == "user";
        let is_system = self.message.role == "system";

        let font_size = self.font_size;
        let content_for_copy = self.message.content.clone();
        let theme_muted = self.theme.text_muted;
        let theme_primary = self.theme.text_primary;
        let message_id = self.index;

        if is_system {
            // System messages - muted, with # prefix
            let content_clone = content_for_copy.clone();
            div()
                .id(("msg", message_id))
                .group("message")
                .flex()
                .px_4()
                .py_1()
                .child(
                    div()
                        .flex()
                        .flex_1()
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
                                .min_w(px(0.0))
                                .text_color(self.theme.text_muted)
                                .text_size(px(font_size - 1.0))
                                .italic()
                                .whitespace_normal()
                                .child(self.message.content.clone())
                        )
                )
                .child(
                    div()
                        .text_size(px(font_size - 2.0))
                        .text_color(theme_muted)
                        .cursor_pointer()
                        .opacity(0.0)
                        .group_hover("message", |s| s.opacity(1.0))
                        .hover(|s| s.text_color(theme_primary))
                        .child("[copy]")
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(content_clone.clone()));
                        })
                )
        } else if is_user {
            // User message - with > prompt prefix
            let has_attachments = !self.message.attachments.is_empty();
            let attachments = self.message.attachments.clone();
            let theme_for_attachments = self.theme.clone();
            let content_clone = content_for_copy.clone();

            div()
                .id(("msg", message_id))
                .group("message")
                .flex()
                .flex_col()
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
                                .min_w(px(0.0))
                                .text_color(self.theme.text_primary)
                                .text_size(px(font_size))
                                .whitespace_normal()
                                .child(self.message.content.clone())
                        )
                        .child(
                            div()
                                .text_size(px(font_size - 2.0))
                                .text_color(theme_muted)
                                .cursor_pointer()
                                .opacity(0.0)
                                .group_hover("message", |s| s.opacity(1.0))
                                .hover(|s| s.text_color(theme_primary))
                                .child("[copy]")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(content_clone.clone()));
                                })
                        )
                )
                // Show attachments if present
                .when(has_attachments, |this| {
                    this.child(
                        div()
                            .pl_4()
                            .child(render_message_attachments(&attachments, &theme_for_attachments, font_size))
                    )
                })
        } else {
            // Assistant message - with markdown rendering
            let content = self.message.content.clone();
            let theme_clone = self.theme.clone();
            let content_clone = content_for_copy.clone();

            div()
                .id(("msg", message_id))
                .group("message")
                .w_full()
                .flex()
                .flex_col()
                .px_4()
                .py_1()
                .gap_0()
                .overflow_hidden()
                // Model info line with copy button (if available)
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
                            .flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(font_size - 2.0))
                                    .text_color(self.theme.text_muted)
                                    .child(format!("--- {} ({}) ---", model_name, provider_name))
                            )
                            .child(
                                div()
                                    .text_size(px(font_size - 2.0))
                                    .text_color(theme_muted)
                                    .cursor_pointer()
                                    .opacity(0.0)
                                    .group_hover("message", |s| s.opacity(1.0))
                                    .hover(|s| s.text_color(theme_primary))
                                    .child("[copy]")
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(content_clone.clone()));
                                    })
                            )
                    )
                })
                // Fallback copy button for messages without model info
                .when(self.message.provider.is_none() || self.message.model.is_none(), |this| {
                    let content_clone2 = content_for_copy.clone();
                    this.child(
                        div()
                            .flex()
                            .justify_end()
                            .child(
                                div()
                                    .text_size(px(font_size - 2.0))
                                    .text_color(theme_muted)
                                    .cursor_pointer()
                                    .opacity(0.0)
                                    .group_hover("message", |s| s.opacity(1.0))
                                    .hover(|s| s.text_color(theme_primary))
                                    .child("[copy]")
                                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(content_clone2.clone()));
                                    })
                            )
                    )
                })
                // Render content with markdown
                .child(render_markdown(&content, &theme_clone, font_size))
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
                .flex_wrap()
                .text_color(theme.text_secondary)
                .text_size(px(font_size))
                .whitespace_normal()
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
