// Chat View - Main chat interface
use gpui::*;
use gpui::prelude::*;
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use gpui_component::input::{InputState, Input, InputEvent};
use std::collections::HashSet;
use crate::state::AppState;
use crate::storage::Storage;
use crate::components::chat_header::ChatHeader;
use crate::components::chat_input::ChatInput;
use crate::components::message_bubble::{MessageBubble, render_streaming_message};
use crate::types::Message;
use crate::gpui_theme::GpuiTheme;

/// A "turn" groups a user message with its associated responses
struct Turn<'a> {
    user_message: &'a Message,
    user_message_index: usize,
    search_message: Option<&'a Message>,
    assistant_messages: Vec<(&'a Message, usize)>,
}

/// Generate a short title from user message content
fn generate_turn_title(content: &str) -> String {
    let content = content.trim();
    if content.is_empty() {
        return "New message".to_string();
    }
    
    // Find first sentence end or take first 60 chars
    let sentence_end = content.find(|c| c == '.' || c == '!' || c == '?')
        .map(|i| i + 1);
    
    let end = match sentence_end {
        Some(i) if i <= 60 => i,
        _ => {
            // Take up to 60 chars, but don't break mid-word
            let chars: String = content.chars().take(60).collect();
            if content.len() > 60 {
                // Find last space to avoid breaking words
                chars.rfind(' ').unwrap_or(chars.len())
            } else {
                chars.len()
            }
        }
    };
    
    let title: String = content.chars().take(end).collect();
    
    if content.len() > end {
        format!("{}...", title.trim_end())
    } else {
        title
    }
}

/// Group messages into turns (user message + associated responses)
fn group_messages_into_turns(messages: &[Message]) -> Vec<Turn<'_>> {
    let mut turns: Vec<Turn<'_>> = Vec::new();
    let mut current_turn: Option<Turn<'_>> = None;
    
    for (idx, msg) in messages.iter().enumerate() {
        match msg.role.as_str() {
            "user" => {
                // Save previous turn if exists
                if let Some(turn) = current_turn.take() {
                    turns.push(turn);
                }
                // Start new turn
                current_turn = Some(Turn {
                    user_message: msg,
                    user_message_index: idx,
                    search_message: None,
                    assistant_messages: Vec::new(),
                });
            }
            "system" if msg.search_results.is_some() => {
                // Add search results to current turn
                if let Some(ref mut turn) = current_turn {
                    turn.search_message = Some(msg);
                }
            }
            "assistant" => {
                // Add assistant message to current turn
                if let Some(ref mut turn) = current_turn {
                    turn.assistant_messages.push((msg, idx));
                }
            }
            _ => {}
        }
    }
    
    // Don't forget the last turn
    if let Some(turn) = current_turn {
        turns.push(turn);
    }
    
    turns
}

pub struct ChatView {
    state: Entity<AppState>,
    header: Entity<ChatHeader>,
    input: Entity<ChatInput>,
    expanded_search_indices: HashSet<usize>,
    scroll_handle: ScrollHandle,
    // Edit state
    editing_message_index: Option<usize>,
    edit_input: Option<Entity<InputState>>,
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
            scroll_handle: ScrollHandle::default(),
            editing_message_index: None,
            edit_input: None,
        }
    }

    fn ensure_edit_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.edit_input.is_none() {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .multi_line()
                    .auto_grow(1, 10)
            });

            // Subscribe to input events for Enter key handling
            cx.subscribe_in(&input, window, Self::handle_edit_input_event)
                .detach();

            self.edit_input = Some(input);
        }
    }

    fn handle_edit_input_event(
        &mut self,
        _input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::PressEnter { secondary } => {
                if !secondary {
                    // Enter without Shift = save
                    self.save_edit_and_resubmit(window, cx);
                }
            }
            _ => {}
        }
    }

    fn start_editing(&mut self, index: usize, content: String, window: &mut Window, cx: &mut Context<Self>) {
        self.ensure_edit_input(window, cx);
        if let Some(input) = &self.edit_input {
            input.update(cx, |input, cx| {
                input.set_value(&content, window, cx);
            });
        }
        self.editing_message_index = Some(index);
        cx.notify();
    }

    fn cancel_editing(&mut self, cx: &mut Context<Self>) {
        self.editing_message_index = None;
        cx.notify();
    }

    fn save_edit_and_resubmit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.editing_message_index else { return };
        let Some(input) = &self.edit_input else { return };

        let new_content = input.read(cx).value().to_string();
        let new_content = new_content.trim().to_string();

        if new_content.is_empty() {
            self.cancel_editing(cx);
            return;
        }

        // Update message and prune subsequent messages
        self.state.update(cx, |state, _| {
            if let Some(chat_id) = &state.current_chat_id {
                if let Some(chat) = state.chats.get_mut(chat_id) {
                    if index < chat.messages.len() {
                        chat.messages[index].content = new_content;
                        chat.messages[index].timestamp = chrono::Utc::now();
                        // Prune all messages after the edited one
                        chat.messages.truncate(index + 1);
                        chat.updated_at = chrono::Utc::now();
                    }

                    // Save to storage
                    if let Err(e) = Storage::save_chats(&state.chats) {
                        eprintln!("Failed to save chats after edit: {}", e);
                    }
                }
            }
        });

        self.editing_message_index = None;

        // Trigger resubmission via ChatInput
        self.input.update(cx, |input, cx| {
            input.resubmit_chat(window, cx);
        });

        cx.notify();
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
                                                    .whitespace_normal()
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
                                                .whitespace_normal()
                                                .child(content)
                                        )
                                    })
                                    .into_any_element()
                            }).collect::<Vec<_>>()
                        )
                )
            })
    }

    fn render_turn(
        &mut self,
        turn_idx: usize,
        turn: &Turn,
        theme: &GpuiTheme,
        font_size: f32,
        editing_index: Option<usize>,
        expanded_indices: &HashSet<usize>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let title = generate_turn_title(&turn.user_message.content);
        let user_idx = turn.user_message_index;
        let is_editing = Some(user_idx) == editing_index;
        
        // Clone data needed for rendering
        let user_message = turn.user_message.clone();
        let search_message = turn.search_message.cloned();
        let assistant_messages: Vec<(Message, usize)> = turn.assistant_messages
            .iter()
            .map(|(m, idx)| ((*m).clone(), *idx))
            .collect();
        let search_idx = user_idx; // Use user message index for search toggle
        let is_search_expanded = expanded_indices.contains(&search_idx);
        
        div()
            .id(("turn", turn_idx))
            .flex()
            .flex_col()
            .w_full()
            // Turn header with title
            .child(
                div()
                    .id(("turn_header", turn_idx))
                    .bg(theme.background)
                    .pt_2()
                    .pb_1()
                    .px_4()
                    .child(
                        div()
                            .text_size(px(font_size))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_primary)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .child(title)
                    )
            )
            // Turn content
            .child(
                div()
                    .flex()
                    .flex_col()
                    // User message (editable)
                    .child(
                        if is_editing {
                            self.render_edit_mode(user_idx, theme, font_size, cx).into_any_element()
                        } else {
                            self.render_user_message_with_edit(user_idx, &user_message, theme, font_size, cx).into_any_element()
                        }
                    )
                    // Search results (if any)
                    .when_some(search_message, |this, search_msg| {
                        this.child(self.render_search_results_element(search_idx, &search_msg, is_search_expanded, cx))
                    })
                    // Assistant messages
                    .children(
                        assistant_messages.iter().map(|(msg, idx)| {
                            MessageBubble::new(msg.clone(), theme.clone(), font_size)
                                .with_index(*idx)
                                .render_element()
                                .into_any_element()
                        })
                    )
            )
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check if we need to scroll to bottom
        let should_scroll = self.state.read(cx).scroll_to_bottom;
        if should_scroll {
            self.scroll_handle.scroll_to_bottom();
            self.state.update(cx, |s, _| s.scroll_to_bottom = false);
        }

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
        let editing_index = self.editing_message_index;

        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .child(
                if let Some(messages) = messages {
                    // Group messages into turns (user message + responses)
                    let turns = group_messages_into_turns(&messages);
                    
                    // Build turn elements
                    let turn_elements: Vec<AnyElement> = turns.iter().enumerate().map(|(turn_idx, turn)| {
                        self.render_turn(turn_idx, turn, &theme, font_size, editing_index, &expanded_indices, cx)
                            .into_any_element()
                    }).collect();

                    div()
                        .id("messages_scroll")
                        .flex()
                        .flex_col()
                        .flex_1()
                        .pr_4()
                        .gap_4()  // Gap between turns
                        .track_scroll(&self.scroll_handle)
                        .scrollable(ScrollbarAxis::Vertical)
                        .children(turn_elements)
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

    fn render_user_message_with_edit(
        &mut self,
        idx: usize,
        msg: &Message,
        theme: &GpuiTheme,
        font_size: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let content = msg.content.clone();
        let content_for_edit = content.clone();
        let content_for_copy = content.clone();
        let has_attachments = !msg.attachments.is_empty();
        let attachments = msg.attachments.clone();
        let theme_for_attachments = theme.clone();
        let theme_muted = theme.text_muted;
        let theme_primary = theme.text_primary;
        let theme_accent = theme.accent;

        div()
            .id(("msg", idx))
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
                            .text_color(theme_accent)
                            .text_size(px(font_size))
                            .font_weight(FontWeight::BOLD)
                            .child(">")
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_color(theme_primary)
                            .text_size(px(font_size))
                            .whitespace_normal()
                            .child(content)
                    )
                    // Edit button
                    .child(
                        div()
                            .id(("edit_btn", idx))
                            .text_size(px(font_size - 2.0))
                            .text_color(theme_muted)
                            .cursor_pointer()
                            .opacity(0.0)
                            .group_hover("message", |s| s.opacity(1.0))
                            .hover(|s| s.text_color(theme_primary))
                            .child("[edit]")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.start_editing(idx, content_for_edit.clone(), window, cx);
                            }))
                    )
                    // Copy button
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
                                cx.write_to_clipboard(ClipboardItem::new_string(content_for_copy.clone()));
                            })
                    )
            )
            // Show attachments if present
            .when(has_attachments, |this| {
                this.child(
                    div()
                        .pl_4()
                        .child(crate::components::attachment_preview::render_message_attachments(&attachments, &theme_for_attachments, font_size))
                )
            })
    }

    fn render_edit_mode(
        &mut self,
        idx: usize,
        theme: &GpuiTheme,
        font_size: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let edit_input = self.edit_input.clone();
        let theme_muted = theme.text_muted;
        let theme_primary = theme.text_primary;
        let theme_accent = theme.accent;
        let theme_danger = theme.danger;

        div()
            .id(("msg_edit", idx))
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
                            .text_color(theme_accent)
                            .text_size(px(font_size))
                            .font_weight(FontWeight::BOLD)
                            .child(">")
                    )
                    .child(
                        div()
                            .flex_1()
                            .when_some(edit_input, |this, input| {
                                this.child(
                                    Input::new(&input)
                                        .cleanable(false)
                                        .appearance(false)
                                        .text_size(px(font_size))
                                        .text_color(theme_primary)
                                )
                            })
                    )
                    // Save button
                    .child(
                        div()
                            .id(("save_edit_btn", idx))
                            .text_size(px(font_size - 2.0))
                            .text_color(theme_muted)
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme_accent))
                            .child("[save]")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_edit_and_resubmit(window, cx);
                            }))
                    )
                    // Cancel button
                    .child(
                        div()
                            .id(("cancel_edit_btn", idx))
                            .text_size(px(font_size - 2.0))
                            .text_color(theme_muted)
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme_danger))
                            .child("[cancel]")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cancel_editing(cx);
                            }))
                    )
            )
            // Warning about regeneration
            .child(
                div()
                    .pl_4()
                    .text_size(px(font_size - 2.0))
                    .text_color(theme_muted)
                    .italic()
                    .child("# saving will regenerate the response")
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
