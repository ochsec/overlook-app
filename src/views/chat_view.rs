// Chat View - Main chat interface
use gpui::*;
use gpui::prelude::*;
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use std::collections::HashSet;
use crate::state::AppState;
use crate::components::chat_header::ChatHeader;
use crate::components::chat_input::ChatInput;
use crate::components::message_bubble::{MessageBubble, render_streaming_message};
use crate::types::Message;

pub struct ChatView {
    state: Entity<AppState>,
    header: Entity<ChatHeader>,
    input: Entity<ChatInput>,
    expanded_search_indices: HashSet<usize>,
}

impl ChatView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let header = cx.new(|_| ChatHeader::new(state.clone()));
        let input = cx.new(|cx| ChatInput::new(state.clone(), window, cx));
        Self {
            state,
            header,
            input,
            expanded_search_indices: HashSet::new(),
        }
    }

    fn toggle_search_expanded(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.expanded_search_indices.contains(&index) {
            self.expanded_search_indices.remove(&index);
        } else {
            self.expanded_search_indices.insert(index);
        }
        cx.notify();
    }

    fn render_search_results_element(
        &self,
        idx: usize,
        msg: &Message,
        is_expanded: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let state = self.state.read(cx);
        let theme = state.theme.clone();
        let font_size = state.font_size;

        let results = msg.search_results.as_ref().unwrap();
        let result_count = results.len();

        let toggle_label = if is_expanded {
            format!("# Web Search ({} results) [-]", result_count)
        } else {
            format!("# Web Search ({} results) [+]", result_count)
        };

        let results_clone = results.clone();

        div()
            .flex()
            .flex_col()
            .px_4()
            .py_1()
            // Header row (always visible, clickable)
            .child(
                div()
                    .id(("search_toggle", idx))
                    .flex()
                    .cursor_pointer()
                    .text_size(px(font_size))
                    .text_color(theme.text_muted)
                    .hover(|s| s.text_color(theme.accent))
                    .child(toggle_label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_search_expanded(idx, cx);
                    }))
            )
            // Expanded content
            .when(is_expanded, |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .pl_4()
                        .gap_1()
                        .mt_1()
                        .children(
                            results_clone.iter().enumerate().map(|(i, result)| {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .py_1()
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .child(
                                        // Title with index
                                        div()
                                            .flex()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(px(font_size - 2.0))
                                                    .text_color(theme.text_muted)
                                                    .child(format!("[{}]", i + 1))
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(font_size - 1.0))
                                                    .text_color(theme.accent)
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .child(result.title.clone())
                                            )
                                    )
                                    .child(
                                        // URL
                                        div()
                                            .text_size(px(font_size - 3.0))
                                            .text_color(theme.text_muted)
                                            .child(result.url.clone())
                                    )
                                    .when(!result.content.is_empty(), |el| {
                                        // Content snippet (truncated)
                                        let content = if result.content.len() > 200 {
                                            format!("{}...", &result.content[..200])
                                        } else {
                                            result.content.clone()
                                        };
                                        el.child(
                                            div()
                                                .text_size(px(font_size - 2.0))
                                                .text_color(theme.text_secondary)
                                                .child(content)
                                        )
                                    })
                                    .into_any_element()
                            }).collect::<Vec<_>>()
                        )
                )
            })
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(state.theme.background)
            .text_color(state.theme.text_primary)
            // Header
            .child(self.header.clone())
            // Messages area
            .child(self.render_messages_area(cx))
            // Input area
            .child(self.input.clone())
    }
}

impl ChatView {
    fn render_messages_area(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        // First extract all needed data from state
        let (messages, theme, font_size, streaming_active, streaming_message, stream_completed, sending_message) = {
            let state = self.state.read(cx);
            if let Some(chat_id) = &state.current_chat_id {
                if let Some(chat) = state.chats.get(chat_id) {
                    (
                        Some(chat.messages.clone()),
                        state.theme.clone(),
                        state.font_size,
                        state.streaming_active,
                        state.streaming_message.clone(),
                        state.stream_completed,
                        state.sending_message,
                    )
                } else {
                    (None, state.theme.clone(), state.font_size, false, String::new(), false, false)
                }
            } else {
                (None, state.theme.clone(), state.font_size, false, String::new(), false, false)
            }
        };

        let expanded_indices = self.expanded_search_indices.clone();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .child(
                if let Some(messages) = messages {
                    // Pre-build message elements
                    let message_elements: Vec<AnyElement> = messages.iter().enumerate().map(|(idx, msg)| {
                        if msg.search_results.is_some() {
                            let is_expanded = expanded_indices.contains(&idx);
                            self.render_search_results_element(idx, msg, is_expanded, cx).into_any_element()
                        } else {
                            MessageBubble::new(msg.clone(), theme.clone(), font_size).render_element().into_any_element()
                        }
                    }).collect();

                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .scrollable(ScrollbarAxis::Vertical)
                        .children(message_elements)
                        // Streaming message
                        .when(streaming_active && !streaming_message.is_empty(), |this| {
                            this.child(
                                render_streaming_message(&streaming_message, &theme, font_size, !stream_completed)
                            )
                        })
                        // Status indicators
                        .when(streaming_active && !stream_completed, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .px_4()
                                    .py_1()
                                    .text_color(theme.text_muted)
                                    .text_size(px(font_size - 2.0))
                                    .child("# streaming...")
                            )
                        })
                        .when(sending_message && !streaming_active, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .px_4()
                                    .py_1()
                                    .text_color(theme.text_muted)
                                    .text_size(px(font_size - 2.0))
                                    .child("# waiting for response...")
                            )
                        })
                        .into_any_element()
                } else {
                    self.render_welcome_message_element(&theme, font_size).into_any_element()
                }
            )
    }

    fn render_welcome_message_element(&self, theme: &crate::gpui_theme::GpuiTheme, font_size: f32) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .p_4()
            .gap_1()
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(theme.text_muted)
                    .child("# overlook v0.1.0")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(theme.text_muted)
                    .child("#")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(theme.text_muted)
                    .child("# Select a model and start chatting,")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(theme.text_muted)
                    .child("# or create a new conversation from the sidebar.")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(theme.text_muted)
                    .child("#")
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(font_size))
                            .text_color(theme.accent)
                            .font_weight(FontWeight::BOLD)
                            .child(">")
                    )
                    .child(
                        div()
                            .text_size(px(font_size))
                            .text_color(theme.text_muted)
                            .child("_")
                    )
            )
    }
}
