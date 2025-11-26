// Chat View - Main chat interface
use gpui::*;
use gpui::prelude::*;
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use crate::state::AppState;
use crate::components::chat_header::ChatHeader;
use crate::components::chat_input::ChatInput;
use crate::components::message_bubble::{MessageBubble, render_streaming_message};

pub struct ChatView {
    state: Entity<AppState>,
    header: Entity<ChatHeader>,
    input: Entity<ChatInput>,
}

impl ChatView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let header = cx.new(|_| ChatHeader::new(state.clone()));
        let input = cx.new(|cx| ChatInput::new(state.clone(), window, cx));
        Self { state, header, input }
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
    fn render_messages_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .child(
                if let Some(chat_id) = &state.current_chat_id {
                    if let Some(chat) = state.chats.get(chat_id) {
                        // Render chat messages
                        let messages: Vec<_> = chat.messages.iter().cloned().collect();
                        let theme = state.theme.clone();
                        let font_size = state.font_size;
                        let streaming_active = state.streaming_active;
                        let streaming_message = state.streaming_message.clone();
                        let stream_completed = state.stream_completed;

                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .scrollable(ScrollbarAxis::Vertical)
                            .children(
                                messages.into_iter().map(|msg| {
                                    MessageBubble::new(msg, theme.clone(), font_size).render_element().into_any_element()
                                }).collect::<Vec<_>>()
                            )
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
                            .when(state.sending_message && !streaming_active, |this| {
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
                        self.render_welcome_message(state).into_any_element()
                    }
                } else {
                    self.render_welcome_message(state).into_any_element()
                }
            )
    }

    fn render_welcome_message(&self, state: &AppState) -> impl IntoElement {
        let font_size = state.font_size;
        div()
            .flex()
            .flex_col()
            .flex_1()
            .p_4()
            .gap_1()
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(state.theme.text_muted)
                    .child("# overlook v0.1.0")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(state.theme.text_muted)
                    .child("#")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(state.theme.text_muted)
                    .child("# Select a model and start chatting,")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(state.theme.text_muted)
                    .child("# or create a new conversation from the sidebar.")
            )
            .child(
                div()
                    .text_size(px(font_size))
                    .text_color(state.theme.text_muted)
                    .child("#")
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(font_size))
                            .text_color(state.theme.accent)
                            .font_weight(FontWeight::BOLD)
                            .child(">")
                    )
                    .child(
                        div()
                            .text_size(px(font_size))
                            .text_color(state.theme.text_muted)
                            .child("_")
                    )
            )
    }
}
