// Chat Input Component - Message input with attachments
use crate::api::{run_on_tokio_async, ApiClient};
use crate::attachments::AttachmentProcessor;
use crate::state::AppState;
use crate::storage::Storage;
use crate::types::{Attachment, Chat, Message};
use chrono::Utc;
use gpui::prelude::*;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use std::time::Duration;

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
            let messages = state
                .current_chat_id
                .as_ref()
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
            .filter_map(|path| match AttachmentProcessor::process_file(path) {
                Ok(attachment) => Some(attachment),
                Err(e) => {
                    eprintln!("Failed to process attachment {}: {}", path, e);
                    None
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
            tool_calls: None,
            tool_result: None,
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
                        title: message_text.chars().take(30).collect::<String>()
                            + if message_text.len() > 30 { "..." } else { "" },
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

        // Prepend the custom system prompt (if set) as a system-role message
        let system_prompt = state_model.read(cx).api_keys.system_prompt.clone();
        if !system_prompt.trim().is_empty() {
            api_messages.insert(
                0,
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                    timestamp: Utc::now(),
                    attachments: Vec::new(),
                    provider: None,
                    model: None,
                    search_results: None,
                    tool_calls: None,
                    tool_result: None,
                    title: None,
                },
            );
        }

        // Check if streaming is enabled
        let streaming_enabled = state_model.read(cx).streaming_enabled;

        if streaming_enabled {
            Self::send_streaming_message(
                state_model,
                chat_id,
                model,
                provider,
                api_key,
                api_messages,
                message_text,
                cx,
            );
        } else {
            Self::send_non_streaming_message(
                state_model,
                chat_id,
                model,
                provider,
                api_key,
                api_messages,
                cx,
            );
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

            let messages = state
                .chats
                .get(&chat_id)
                .map(|chat| chat.messages.clone())
                .unwrap_or_default();

            if messages.is_empty() {
                println!("No messages in chat");
                return;
            }

            // Get the last user message for potential web search
            let last_user_query = messages
                .iter()
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

        // Prepend the custom system prompt (if set) as a system-role message
        let system_prompt = state_model.read(cx).api_keys.system_prompt.clone();
        let mut api_messages = api_messages;
        if !system_prompt.trim().is_empty() {
            api_messages.insert(
                0,
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                    timestamp: Utc::now(),
                    attachments: Vec::new(),
                    provider: None,
                    model: None,
                    search_results: None,
                    tool_calls: None,
                    tool_result: None,
                    title: None,
                },
            );
        }

        // Set sending state
        state_model.update(cx, |state, _| {
            state.sending_message = true;
        });

        // Check if streaming is enabled
        let streaming_enabled = state_model.read(cx).streaming_enabled;

        if streaming_enabled {
            Self::send_streaming_message(
                state_model,
                chat_id,
                model,
                provider,
                api_key,
                api_messages,
                user_query,
                cx,
            );
        } else {
            Self::send_non_streaming_message(
                state_model,
                chat_id,
                model,
                provider,
                api_key,
                api_messages,
                cx,
            );
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
                                "png", "jpg", "jpeg", "gif", "webp", "bmp", "pdf", "txt", "md",
                                "json", "xml", "csv", "log",
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
                })
                .await;

                // Update state with selected files
                if let Some(paths) = file_paths {
                    cx.update(|cx| {
                        state_model.update(cx, |state, _| {
                            state.selected_attachments.extend(paths);
                        });
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn send_streaming_message(
        state_model: Entity<AppState>,
        chat_id: String,
        model: String,
        provider: crate::types::ProviderKind,
        api_key: String,
        api_messages: Vec<Message>,
        _user_query: String, // Kept for signature compatibility if needed, but we now use MCP
        cx: &mut Context<Self>,
    ) {
        // Set streaming state
        state_model.update(cx, |state, _| {
            state.streaming_active = true;
            state.streaming_message.clear();
            state.stream_completed = false;
        });

        // Get MCP manager on the main thread so tool listing works reliably in the async task
        let mcp_manager = state_model.read(cx).mcp_manager.clone();

        // Spawn task to handle tool-enabled streaming
        let model_for_result = model.clone();
        cx.spawn({
            let state_model = state_model.clone();
            let mcp_manager = mcp_manager;
            let api_key = api_key.clone();
            let model = model.clone();
            async move |_this, cx| {
                let mut current_messages = api_messages;

                loop {
                    // List tools from all connected MCP servers for the LLM to use
                    let tools: Vec<crate::types::Tool> = mcp_manager
                        .list_all_tools()
                        .await
                        .into_iter()
                        .map(|(server_name, tool)| {
                            // Sanitize compound name to only contain [a-zA-Z0-9_-]
                            // Many LLM providers (e.g. Amazon Bedrock) require tool names matching ^[a-zA-Z0-9_-]{1,128}$
                            let safe_server: String = server_name
                                .chars()
                                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                                .collect();
                            let safe_tool: String = tool.name.as_ref()
                                .chars()
                                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                                .collect();
                            let name = format!("{}__{}", safe_server, safe_tool);
                            let description = tool
                                .description
                                .as_deref()
                                .unwrap_or("")
                                .to_string();
                            // Convert MCP input_schema to JSON Value for OpenAI/Anthropic parameters
                            let schema = serde_json::to_value(&*tool.input_schema)
                                .ok()
                                .filter(|v| v.is_object())
                                .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
                            crate::types::Tool {
                                name,
                                server_name,
                                description,
                                schema: Some(schema),
                            }
                        })
                        .collect();

                    println!("[Chat] Sending {} tools to LLM", tools.len());
                    for t in &tools {
                        println!("[Chat]   - {} (server: {})", t.name, t.server_name);
                    }

                    let (tx, rx) = std::sync::mpsc::channel();

                    // Start streaming
                    let api_client_spawn = ApiClient::new();
                    let provider_spawn = provider;
                    let api_key_spawn = api_key.clone();
                    let model_spawn = model.clone();
                    let current_messages_spawn = current_messages.clone();

                    // Convert our tools to API ToolDefinition format
                    let tool_definitions: Vec<crate::api::ToolDefinition> = tools
                        .iter()
                        .map(|tool| crate::api::ToolDefinition {
                            name: tool.name.clone(),
                            server_name: tool.server_name.clone(),
                            description: Some(tool.description.clone()),
                            input_schema: tool.schema.clone().unwrap_or(serde_json::Value::Null),
                        })
                        .collect();

                    run_on_tokio_async(async move {
                        api_client_spawn
                            .stream_message_with_tools(
                                provider_spawn,
                                &api_key_spawn,
                                &model_spawn,
                                &current_messages_spawn,
                                &tool_definitions,
                                tx,
                            )
                            .await;
                    })
                    .await;

                    let mut full_response = String::new();
                    let mut pending_tool_calls: Vec<crate::types::PendingToolCall> = Vec::new();
                    let mut had_error = false;

                    // Poll the channel for events
                    loop {
                        match rx.try_recv() {
                            Ok(event) => match event {
                                crate::api::StreamEvent::Content(chunk) => {
                                    full_response.push_str(&chunk);
                                    let response_so_far = full_response.clone();
                                    cx.update(|cx| {
                                        state_model.update(cx, |state, _| {
                                            state.streaming_message = response_so_far;
                                        });
                                    })
                                    .ok();
                                }
                                crate::api::StreamEvent::ToolCallStart { index, id, name } => {
                                    pending_tool_calls.push(crate::types::PendingToolCall {
                                        index,
                                        id,
                                        name,
                                        arguments: String::new(),
                                        server_name: String::new(), // Will be filled from tools list
                                    });
                                }
                                crate::api::StreamEvent::ToolCallDelta { index, arguments } => {
                                    if let Some(tc) =
                                        pending_tool_calls.iter_mut().find(|tc| tc.index == index)
                                    {
                                        tc.arguments.push_str(&arguments);
                                    }
                                }
                                crate::api::StreamEvent::ToolCallEnd { .. } => {
                                    // Handled after stream ends
                                }
                                crate::api::StreamEvent::Done => break,
                                crate::api::StreamEvent::Error(e) => {
                                    eprintln!("Streaming error: {}", e);
                                    had_error = true;
                                    break;
                                }
                            },
                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                smol::Timer::after(Duration::from_millis(10)).await;
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                        }
                    }

                    if had_error {
                        break;
                    }

                    // If we have tool calls, execute them and continue
                    if !pending_tool_calls.is_empty() {
                        // Create assistant message with tool calls
                        let mut tool_calls = Vec::new();
                        for ptc in &pending_tool_calls {
                            let arguments =
                                serde_json::from_str(&ptc.arguments).unwrap_or_default();

                            // Find server name for this tool
                            let server_name = tools
                                .iter()
                                .find(|t| t.name == ptc.name)
                                .map(|t| t.server_name.clone())
                                .unwrap_or_default();

                            tool_calls.push(crate::types::ToolCall {
                                id: ptc.id.clone(),
                                name: ptc.name.clone(),
                                server_name,
                                arguments: Some(arguments),
                            });
                        }

                        let assistant_message = Message::new_assistant_with_tool_calls(
                            full_response.clone(),
                            provider,
                            model_for_result.clone(),
                            tool_calls.clone(),
                        );

                        // Add to chat and state
                        current_messages.push(assistant_message.clone());
                        cx.update(|cx| {
                            state_model.update(cx, |state, _| {
                                if let Some(chat) = state.chats.get_mut(&chat_id) {
                                    chat.messages.push(assistant_message);
                                }
                                state.streaming_message.clear();
                            });
                        })
                        .ok();

                        // Execute tools
                        let mut results = Vec::new();
                        for ptc in &pending_tool_calls {
                            let arguments =
                                serde_json::from_str(&ptc.arguments).unwrap_or_default();

                            let (server_name, mcp_tool_name) = tools
                                .iter()
                                .find(|t| t.name == ptc.name)
                                .map(|t| {
                                    // Extract short MCP tool name from "server__tool" for call_tool
                                    let short = t.name
                                        .rsplit_once("__")
                                        .map(|(_, n)| n.to_string())
                                        .unwrap_or_else(|| t.name.clone());
                                    (t.server_name.clone(), short)
                                })
                                .unwrap_or_default();

                            println!(
                                "Executing MCP tool: {} on server: {}",
                                mcp_tool_name, server_name
                            );

                            let mcp_mgr = mcp_manager.clone();
                            let srv = server_name.clone();
                            let tool = mcp_tool_name.clone();
                            println!("[Chat] Calling MCP tool: {} on server: {} with args: {:?}", tool, srv, arguments);
                            let result = crate::api::run_on_tokio_async(async move {
                                mcp_mgr.call_tool(&srv, &tool, arguments).await
                            })
                            .await;

                            match result {
                                Ok(content) => {
                                    results.push(crate::types::ToolResult {
                                        tool_call_id: ptc.id.clone(),
                                        content,
                                        is_error: false,
                                    });
                                }
                                Err(e) => {
                                    results.push(crate::types::ToolResult {
                                        tool_call_id: ptc.id.clone(),
                                        content: e.to_string(),
                                        is_error: true,
                                    });
                                }
                            }
                        }

                        // Add tool results to messages
                        for result in results {
                            let result_message = Message::new_tool_result(result);
                            current_messages.push(result_message.clone());
                            cx.update(|cx| {
                                state_model.update(cx, |state, _| {
                                    if let Some(chat) = state.chats.get_mut(&chat_id) {
                                        chat.messages.push(result_message);
                                    }
                                });
                            })
                            .ok();
                        }

                        // Clear pending tool calls and full response before looping for next response
                        pending_tool_calls.clear();
                        full_response.clear();

                        // Loop again to get assistant response for tool results
                        continue;
                    } else {
                        // No tool calls, we're done
                        cx.update(|cx| {
                            state_model.update(cx, |state, _| {
                                state.sending_message = false;
                                state.streaming_active = false;
                                state.stream_completed = true;

                                if !full_response.is_empty() {
                                    let assistant_message = Message::new_assistant(
                                        full_response,
                                        provider,
                                        model_for_result.clone(),
                                    );

                                    if let Some(chat) = state.chats.get_mut(&chat_id) {
                                        chat.messages.push(assistant_message);
                                        chat.updated_at = Utc::now();
                                    }

                                    state.streaming_message.clear();
                                    if let Err(e) = Storage::save_chats(&state.chats) {
                                        eprintln!("Failed to save chats: {}", e);
                                    }
                                }
                            });
                        })
                        .ok();
                        break;
                    }
                }
            }
        })
        .detach();
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
                    api_client
                        .send_chat_message(provider, &api_key, &model, api_messages)
                        .await
                })
                .await;

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
                                    tool_calls: None,
                                    tool_result: None,
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
                })
                .ok();
            }
        })
        .detach();
    }
}

impl Render for ChatInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        let is_sending =
            state.sending_message || (state.streaming_active && !state.stream_completed);
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
        let attachments: Vec<_> = state
            .selected_attachments
            .iter()
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
                    div().flex().gap_2().flex_wrap().children(
                        attachments
                            .iter()
                            .map(|(i, file_name)| {
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
                                            .child(format!("# attached: {}", file_name)),
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
                                            }),
                                    )
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>(),
                    ),
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
                            .child(">"),
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
                            .min_h(px(font_size * 5.0 + 24.0))
                            .child(
                                Input::new(&self.input)
                                    .cleanable(false)
                                    .appearance(false)
                                    .text_size(px(font_size))
                                    .text_color(theme_text_primary)
                                    .line_height(px(font_size + 4.0)),
                            )
                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                                // Handle Shift+Return to insert newline
                                if event.keystroke.key == "enter" && event.keystroke.modifiers.shift
                                {
                                    this.insert_newline(window, cx);
                                }
                            })),
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
                            })),
                    )
                    .child(
                        // Send button (minimal text style)
                        div()
                            .id("send_btn")
                            .text_size(px(font_size - 2.0))
                            .text_color(if is_sending {
                                theme_text_muted
                            } else {
                                theme_text_primary
                            })
                            .cursor_pointer()
                            .when(!is_sending, |this| {
                                this.hover(|s| s.text_color(theme_accent))
                            })
                            .pt_2()
                            .child(send_label)
                            .when(!is_sending, |this| {
                                this.on_click(cx.listener(|this, _, window, cx| {
                                    this.submit_message(window, cx);
                                }))
                            }),
                    ),
            )
    }
}
