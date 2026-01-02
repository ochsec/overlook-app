// Chat Input Component - Message input with attachments
use gpui::*;
use gpui::prelude::*;
use gpui_component::input::{InputState, Input, InputEvent};
use chrono::Utc;
use std::time::Duration;
use crate::api::{ApiClient, run_on_tokio_async, stream_chat_to_channel};
use crate::attachments::AttachmentProcessor;
use crate::state::AppState;
use crate::storage::Storage;
use crate::tools::SearchOrchestrator;
use crate::types::{Attachment, Chat, Message};

pub struct ChatInput {
    state: Entity<AppState>,
    input: Entity<InputState>,
}

impl ChatInput {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .auto_grow(1, 10)
                .placeholder("Type message... (Shift+Return for newline)")
        });

        // Subscribe to input events to handle Enter key
        cx.subscribe_in(&input, window, Self::handle_input_event)
            .detach();

        Self { state, input }
    }

    fn handle_input_event(
        &mut self,
        _input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::PressEnter { secondary } => {
                if !*secondary {
                    // Plain Enter submits the message
                    self.submit_message(window, cx);
                }
                // Shift+Enter is handled via on_key_down handler below
            }
            _ => {}
        }
    }

    fn submit_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state_model = self.state.clone();
        let input_entity = self.input.clone();

        // Check if already sending
        let is_sending = {
            let state = self.state.read(cx);
            state.sending_message || (state.streaming_active && !state.stream_completed)
        };

        if is_sending {
            return;
        }

        // Read message from input
        let message_text = input_entity.read(cx).value().to_string();
        let message_text = message_text.trim().to_string();

        if message_text.is_empty() {
            return;
        }

        // Extract values needed for async task
        let (provider, model, api_key, chat_id, existing_messages, attachment_paths) = {
            let state = state_model.read(cx);
            if state.current_model.is_empty() {
                println!("No model selected");
                return;
            }
            let api_key = state.api_keys.get(state.current_provider).to_string();
            let messages = state.current_chat_id.as_ref()
                .and_then(|id| state.chats.get(id))
                .map(|chat| chat.messages.clone())
                .unwrap_or_default();
            (
                state.current_provider,
                state.current_model.clone(),
                api_key,
                state.current_chat_id.clone(),
                messages,
                state.selected_attachments.clone(),
            )
        };

        // Process attachments from file paths
        let attachments: Vec<Attachment> = attachment_paths
            .iter()
            .filter_map(|path| {
                match AttachmentProcessor::process_file(path) {
                    Ok(attachment) => Some(attachment),
                    Err(e) => {
                        eprintln!("Failed to process attachment {}: {}", path, e);
                        None
                    }
                }
            })
            .collect();

        // Create user message
        let user_message = Message {
            role: "user".to_string(),
            content: message_text.clone(),
            timestamp: Utc::now(),
            attachments,
            provider: Some(provider),
            model: Some(model.clone()),
            search_results: None,
            title: None,
        };

        // Create or update chat
        let chat_id = state_model.update(cx, |state, _| {
            let chat_id = match chat_id {
                Some(id) => id,
                None => {
                    // Create new chat
                    let now = Utc::now();
                    let new_chat = Chat {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: message_text.chars().take(30).collect::<String>() + if message_text.len() > 30 { "..." } else { "" },
                        messages: Vec::new(),
                        model: model.clone(),
                        provider,
                        project_id: state.current_project_id.clone(),
                        created_at: now,
                        updated_at: now,
                    };
                    let id = new_chat.id.clone();
                    state.chats.insert(id.clone(), new_chat);
                    state.current_chat_id = Some(id.clone());
                    id
                }
            };

            // Add user message to chat
            if let Some(chat) = state.chats.get_mut(&chat_id) {
                chat.messages.push(user_message.clone());
                chat.updated_at = Utc::now();
            }

            // Save chats immediately after adding user message
            if let Err(e) = Storage::save_chats(&state.chats) {
                eprintln!("Failed to save chats: {}", e);
            }

            // Set sending state and clear attachments
            state.sending_message = true;
            state.scroll_to_bottom = true;
            state.selected_attachments.clear();
            chat_id
        });

        // Clear the input field
        input_entity.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });

        // Build messages for API (include the new user message)
        let mut api_messages = existing_messages;
        api_messages.push(user_message);

        // Check if streaming is enabled
        let streaming_enabled = state_model.read(cx).streaming_enabled;

        if streaming_enabled {
            Self::send_streaming_message(state_model, chat_id, model, provider, api_key, api_messages, message_text, cx);
        } else {
            Self::send_non_streaming_message(state_model, chat_id, model, provider, api_key, api_messages, cx);
        }
    }

    /// Resubmit the current chat to get a new AI response.
    /// Used after editing a message to regenerate the response.
    pub fn resubmit_chat(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let state_model = self.state.clone();

        // Check if already sending
        let is_sending = {
            let state = self.state.read(cx);
            state.sending_message || (state.streaming_active && !state.stream_completed)
        };

        if is_sending {
            return;
        }

        // Extract values needed for async task
        let (provider, model, api_key, chat_id, api_messages, user_query) = {
            let state = state_model.read(cx);
            if state.current_model.is_empty() {
                println!("No model selected");
                return;
            }

            let chat_id = match &state.current_chat_id {
                Some(id) => id.clone(),
                None => {
                    println!("No chat selected");
                    return;
                }
            };

            let messages = state.chats.get(&chat_id)
                .map(|chat| chat.messages.clone())
                .unwrap_or_default();

            if messages.is_empty() {
                println!("No messages in chat");
                return;
            }

            // Get the last user message for potential web search
            let last_user_query = messages.iter()
                .rev()
                .find(|m| m.role == "user")
                .map(|m| m.content.clone())
                .unwrap_or_default();

            let api_key = state.api_keys.get(state.current_provider).to_string();

            (
                state.current_provider,
                state.current_model.clone(),
                api_key,
                chat_id,
                messages,
                last_user_query,
            )
        };

        // Set sending state
        state_model.update(cx, |state, _| {
            state.sending_message = true;
        });

        // Check if streaming is enabled
        let streaming_enabled = state_model.read(cx).streaming_enabled;

        if streaming_enabled {
            Self::send_streaming_message(state_model, chat_id, model, provider, api_key, api_messages, user_query, cx);
        } else {
            Self::send_non_streaming_message(state_model, chat_id, model, provider, api_key, api_messages, cx);
        }
    }

    fn insert_newline(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input_entity = self.input.clone();
        
        input_entity.update(cx, |input_state, cx| {
            // Insert a newline at the current cursor position
            input_state.insert("\n", window, cx);
        });
    }

    fn open_file_picker(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let state_model = self.state.clone();

        cx.spawn({
            let state_model = state_model.clone();
            async move |_this, cx| {
                // Run file picker on tokio runtime
                let file_paths = run_on_tokio_async(async move {
                    let files = rfd::AsyncFileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                        .add_filter("PDFs", &["pdf"])
                        .add_filter("Text files", &["txt", "md", "json", "xml", "csv", "log"])
                        .add_filter(
                            "All supported",
                            &[
                                "png", "jpg", "jpeg", "gif", "webp", "bmp", "pdf", "txt", "md", "json",
                                "xml", "csv", "log",
                            ],
                        )
                        .pick_files()
                        .await;

                    // Convert to file paths, filtering to supported types
                    files.map(|files| {
                        files
                            .into_iter()
                            .map(|file| file.path().to_string_lossy().to_string())
                            .filter(|path| {
                                crate::attachments::AttachmentProcessor::is_supported_file(path)
                            })
                            .collect::<Vec<String>>()
                    })
                }).await;

                // Update state with selected files
                if let Some(paths) = file_paths {
                    cx.update(|cx| {
                        state_model.update(cx, |state, _| {
                            state.selected_attachments.extend(paths);
                        });
                    }).ok();
                }
            }
        }).detach();
    }

    fn send_streaming_message(
        state_model: Entity<AppState>,
        chat_id: String,
        model: String,
        provider: crate::types::ProviderKind,
        api_key: String,
        api_messages: Vec<Message>,
        user_query: String,
        cx: &mut Context<Self>,
    ) {
        // Set streaming state
        state_model.update(cx, |state, _| {
            state.streaming_active = true;
            state.streaming_message.clear();
            state.stream_completed = false;
        });

        // Spawn task to handle search + streaming
        let model_for_result = model.clone();
        cx.spawn({
            let state_model = state_model.clone();
            let api_key = api_key.clone();
            let model = model.clone();
            async move |_this, cx| {
                // Check if web search is configured and perform search if needed
                let search_orchestrator = SearchOrchestrator::new();
                let mut final_api_messages = api_messages.clone();

                // Try to perform web search if configured
                let search_result = run_on_tokio_async({
                    let user_query = user_query.clone();
                    let api_key = api_key.clone();
                    let model = model.clone();
                    async move {
                        // Check if search is configured
                        if std::env::var("SEARXNG_URL").is_err() {
                            return None;
                        }

                        println!("Web search is configured, checking if search is needed...");

                        // First, ask the LLM if it needs to search
                        let decision_prompt = SearchOrchestrator::create_search_decision_message(&user_query);
                        let decision_message = Message {
                            role: "user".to_string(),
                            content: decision_prompt,
                            timestamp: Utc::now(),
                            attachments: Vec::new(),
                            provider: Some(provider),
                            model: Some(model.clone()),
                            search_results: None,
                            title: None,
                        };

                        let api_client = ApiClient::new();
                        match api_client.send_chat_message(
                            provider,
                            &api_key,
                            &model,
                            vec![decision_message],
                        ).await {
                            Ok(response) => {
                                println!("Search decision response: {}", response);
                                match SearchOrchestrator::parse_search_decision(&response) {
                                    Ok(decision) => {
                                        if decision.search {
                                            if let Some(query) = decision.query {
                                                println!("Searching for: {}", query);
                                                // Perform the search
                                                match search_orchestrator.search_and_format(&query).await {
                                                    Ok((formatted, results)) => {
                                                        println!("Search completed successfully");
                                                        return Some((formatted, results));
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Search failed: {}", e);
                                                    }
                                                }
                                            }
                                        } else {
                                            println!("LLM decided search not needed");
                                        }
                                    }
                                    Err(e) => eprintln!("Failed to parse search decision: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Failed to get search decision: {}", e),
                        }
                        None
                    }
                }).await;

                // If we got search results, add them as a system message
                if let Some((search_context, search_results)) = search_result {
                    let search_message = Message {
                        role: "system".to_string(),
                        content: search_context,
                        timestamp: Utc::now(),
                        attachments: Vec::new(),
                        provider: Some(provider),
                        model: Some(model.clone()),
                        search_results: if search_results.is_empty() { None } else { Some(search_results) },
                        title: None,
                    };
                    // Insert at the beginning
                    final_api_messages.insert(0, search_message);
                }

                // Now start streaming with the (possibly augmented) messages
                let (tx, rx) = std::sync::mpsc::channel();

                // Start streaming in background thread
                stream_chat_to_channel(
                    provider,
                    api_key.clone(),
                    model.clone(),
                    final_api_messages,
                    tx,
                );

                let mut full_response = String::new();
                let mut had_error = false;

                // Poll the channel for chunks
                loop {
                    match rx.try_recv() {
                        Ok(Ok(chunk)) => {
                            full_response.push_str(&chunk);
                            let response_so_far = full_response.clone();
                            cx.update(|cx| {
                                state_model.update(cx, |state, _| {
                                    state.streaming_message = response_so_far;
                                });
                            }).ok();
                        }
                        Ok(Err(e)) => {
                            eprintln!("Streaming error: {}", e);
                            had_error = true;
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            // Wait a bit and check again
                            smol::Timer::after(Duration::from_millis(10)).await;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            // Stream finished
                            break;
                        }
                    }
                }

                // Finalize streaming
                let final_response = full_response;
                cx.update(|cx| {
                    state_model.update(cx, |state, _| {
                        state.sending_message = false;
                        state.streaming_active = false;
                        state.stream_completed = true;

                        if !had_error && !final_response.is_empty() {
                            // Create assistant message from streamed content
                            let assistant_message = Message {
                                role: "assistant".to_string(),
                                content: final_response,
                                timestamp: Utc::now(),
                                attachments: Vec::new(),
                                provider: Some(provider),
                                model: Some(model_for_result.clone()),
                                search_results: None,
                                title: None,
                            };

                            // Add to chat
                            if let Some(chat) = state.chats.get_mut(&chat_id) {
                                chat.messages.push(assistant_message);
                                chat.updated_at = Utc::now();
                            }

                            // Clear streaming message
                            state.streaming_message.clear();

                            // Save chats to disk
                            if let Err(e) = Storage::save_chats(&state.chats) {
                                eprintln!("Failed to save chats: {}", e);
                            }

                            println!("Streaming completed successfully");
                        }
                    });
                }).ok();
            }
        }).detach();
    }

    fn send_non_streaming_message(
        state_model: Entity<AppState>,
        chat_id: String,
        model: String,
        provider: crate::types::ProviderKind,
        api_key: String,
        api_messages: Vec<Message>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn({
            let state_model = state_model.clone();
            let model_for_result = model.clone();
            async move |_this, cx| {
                // Run reqwest on Tokio runtime (GPUI uses smol)
                let result = run_on_tokio_async(async move {
                    let api_client = ApiClient::new();
                    api_client.send_chat_message(
                        provider,
                        &api_key,
                        &model,
                        api_messages,
                    ).await
                }).await;

                cx.update(|cx| {
                    state_model.update(cx, |state, _| {
                        state.sending_message = false;

                        match result {
                            Ok(response_text) => {
                                // Create assistant message
                                let assistant_message = Message {
                                    role: "assistant".to_string(),
                                    content: response_text,
                                    timestamp: Utc::now(),
                                    attachments: Vec::new(),
                                    provider: Some(provider),
                                    model: Some(model_for_result.clone()),
                                    search_results: None,
                                    title: None,
                                };

                                // Add to chat
                                if let Some(chat) = state.chats.get_mut(&chat_id) {
                                    chat.messages.push(assistant_message);
                                    chat.updated_at = Utc::now();
                                }

                                // Save chats to disk
                                if let Err(e) = Storage::save_chats(&state.chats) {
                                    eprintln!("Failed to save chats: {}", e);
                                }

                                println!("Message sent successfully");
                            }
                            Err(e) => {
                                eprintln!("Failed to send message: {}", e);
                            }
                        }
                    });
                }).ok();
            }
        }).detach();
    }
}

impl Render for ChatInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        let is_sending = state.sending_message || (state.streaming_active && !state.stream_completed);
        let send_label = if state.streaming_active && !state.stream_completed {
            "[streaming]"
        } else if state.sending_message {
            "[sending]"
        } else {
            "[send]"
        };

        // Extract theme values
        let theme_background = state.theme.background;
        let theme_border = state.theme.border;
        let theme_text_primary = state.theme.text_primary;
        let theme_text_muted = state.theme.text_muted;
        let theme_accent = state.theme.accent;
        let font_size = state.font_size;

        // Extract attachment info
        let attachments: Vec<_> = state.selected_attachments.iter()
            .enumerate()
            .map(|(i, path)| {
                let file_name = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                (i, file_name)
            })
            .collect();
        let has_attachments = !attachments.is_empty();

        div()
            .flex()
            .flex_col()
            .px_4()
            .py_2()
            .gap_1()
            .bg(theme_background)
            .border_t_1()
            .border_color(theme_border)
            // Attachments preview (if any)
            .when(has_attachments, |this| {
                this.child(
                    div()
                        .flex()
                        .gap_2()
                        .flex_wrap()
                        .children(
                            attachments.iter().map(|(i, file_name)| {
                                let idx = *i;
                                let state_model = state_model.clone();
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(font_size - 2.0))
                                            .text_color(theme_text_muted)
                                            .child(format!("# attached: {}", file_name))
                                    )
                                    .child(
                                        div()
                                            .text_size(px(font_size - 2.0))
                                            .text_color(theme_text_muted)
                                            .cursor_pointer()
                                            .hover(|s| s.text_color(theme_text_primary))
                                            .child("[x]")
                                            .on_mouse_down(MouseButton::Left, move |_, _, app| {
                                                state_model.update(app, |state, _| {
                                                    if idx < state.selected_attachments.len() {
                                                        state.selected_attachments.remove(idx);
                                                    }
                                                });
                                            })
                                    )
                                    .into_any_element()
                            }).collect::<Vec<_>>()
                        )
                )
            })
            // Input row with terminal prompt style
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    // Prompt indicator
                    .child(
                        div()
                            .text_size(px(font_size))
                            .text_color(theme_accent)
                            .font_weight(FontWeight::BOLD)
                            .pt_2()
                            .child(">")
                    )
                    .child(
                        // Text input area - textarea style with border
                        div()
                            .flex_1()
                            .border_1()
                            .border_color(theme_border)
                            .rounded_md()
                            .px_2()
                            .py_2()
                            .bg(gpui::rgb(0x1a1a1a))
                            .min_h(px(font_size * 3.0 + 16.0))
                            .child(
                                Input::new(&self.input)
                                    .cleanable(false)
                                    .appearance(false)
                                    .text_size(px(font_size))
                                    .text_color(theme_text_primary)
                                    .line_height(px(font_size + 4.0))
                            )
                            .on_key_down(
                                cx.listener(|this, event: &KeyDownEvent, window, cx| {
                                    // Handle Shift+Return to insert newline
                                    if event.keystroke.key == "enter" && event.keystroke.modifiers.shift {
                                        this.insert_newline(window, cx);
                                    }
                                })
                            )
                    )
                    .child(
                        // Attach button (minimal)
                        div()
                            .id("attach_btn")
                            .text_size(px(font_size - 2.0))
                            .text_color(theme_text_muted)
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme_text_primary))
                            .pt_2()
                            .child("[attach]")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_file_picker(window, cx);
                            }))
                    )
                    .child(
                        // Send button (minimal text style)
                        div()
                            .id("send_btn")
                            .text_size(px(font_size - 2.0))
                            .text_color(if is_sending { theme_text_muted } else { theme_text_primary })
                            .cursor_pointer()
                            .when(!is_sending, |this| this.hover(|s| s.text_color(theme_accent)))
                            .pt_2()
                            .child(send_label)
                            .when(!is_sending, |this| {
                                this.on_click(cx.listener(|this, _, window, cx| {
                                    this.submit_message(window, cx);
                                }))
                            })
                    )
            )
    }
}
