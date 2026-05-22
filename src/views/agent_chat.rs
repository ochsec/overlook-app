use crate::api::run_on_tokio_async;
use crate::hermes_api::HermesClient;
use crate::state::AppState;
use crate::types::{Message, ProviderKind};
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::Disableable;

pub struct AgentChatView {
    state: Entity<AppState>,
    session_id: String,
    messages: Vec<Message>,
    input: Entity<InputState>,
    sending: bool,
    error_message: Option<String>,
    scroll_handle: ScrollHandle,
}

impl AgentChatView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .auto_grow(1, 10)
                .placeholder("Type a message to Hermes Agent...")
        });

        cx.subscribe_in(&input, window, Self::handle_input_event)
            .detach();

        Self {
            state,
            session_id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
            input,
            sending: false,
            error_message: None,
            scroll_handle: ScrollHandle::default(),
        }
    }

    /// Build a HermesClient using the current hermes_url and hermes_api_key from settings.
    /// This ensures URL/key changes in Settings take effect immediately.
    fn get_client(&self, cx: &Context<Self>) -> HermesClient {
        let keys = &self.state.read(cx).api_keys;
        HermesClient::with_base_url_and_key(keys.hermes_url.clone(), keys.hermes_api_key.clone())
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
                    self.submit_message(window, cx);
                }
            }
            _ => {}
        }
    }

    fn submit_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sending {
            return;
        }

        let input_text = self.input.read(cx).value().to_string();
        let message = input_text.trim().to_string();
        if message.is_empty() {
            return;
        }

        // Add user message to UI immediately
        let user_msg = Message::new_user(message.clone());
        self.messages.push(user_msg);

        // Clear input field
        self.input.update(cx, |input_state, cx| {
            input_state.set_value("", window, cx);
        });

        self.sending = true;
        self.error_message = None;
        cx.notify();

        // Build client from current settings so URL changes take effect
        let client = self.get_client(cx);
        let session_id = self.session_id.clone();
        let this = cx.entity().clone();

        cx.spawn(async move |_this, cx| {
            // Run the reqwest call on the Tokio runtime (reqwest needs a Tokio reactor)
            let result = run_on_tokio_async(async move {
                client.send_message_async(&message, &session_id).await
            })
            .await;

            cx.update(|cx| {
                this.update(cx, |this, cx| {
                    match result {
                        Ok((response, new_session_id)) => {
                            this.session_id = new_session_id;
                            let assistant_msg = Message::new_assistant(
                                response,
                                ProviderKind::Hermes,
                                "hermes-agent".to_string(),
                            );
                            this.messages.push(assistant_msg);
                        }
                        Err(e) => {
                            this.error_message = Some(format!("Error: {}", e));
                        }
                    }
                    this.sending = false;
                    cx.notify();
                })
            })
            .ok();
        })
        .detach();
    }

    fn clear_chat(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        self.session_id = uuid::Uuid::new_v4().to_string();
        self.error_message = None;
        cx.notify();
    }
}

impl Render for AgentChatView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.state.read(cx).theme;
        let hermes_url = self.state.read(cx).api_keys.hermes_url.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("Hermes Agent"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(theme.text_muted)
                                    .child(format!("Endpoint: {}", hermes_url)),
                            )
                            .child(
                                Button::new("clear_agent_chat")
                                    .label("Clear")
                                    .compact()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_chat(cx);
                                    })),
                            ),
                    ),
            )
            // Chat messages area
            .child(
                div()
                    .id("agent_chat_scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p_4()
                    .gap_2()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .children(self.messages.iter().map(|msg| {
                        let is_user = msg.role == "user";
                        div()
                            .flex()
                            .flex_row()
                            .when(is_user, |div| div.justify_end())
                            .child(
                                div()
                                    .p_3()
                                    .m_1()
                                    .max_w(px(600.0))
                                    .bg(if is_user { theme.accent } else { theme.surface })
                                    .rounded_lg()
                                    .text_color(if is_user { gpui::white() } else { theme.text_primary })
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .opacity(0.7)
                                            .child(if is_user { "You" } else { "Hermes" }),
                                    )
                                    .child(
                                        div().mt_1().child(msg.content.clone()),
                                    ),
                            )
                    }))
                    .when_some(self.error_message.as_ref(), |el, error| {
                        el.child(
                            div()
                                .p_3()
                                .bg(rgb(0xFFE0E0))
                                .text_color(rgb(0xCC0000))
                                .rounded_lg()
                                .child(error.clone()),
                        )
                    })
                    .when(self.sending, |el| {
                        el.child(
                            div()
                                .p_2()
                                .text_color(theme.text_muted)
                                .child("Hermes is thinking..."),
                        )
                    }),
            )
            // Input area
            .child(
                div()
                    .flex()
                    .flex_row()
                    .p_3()
                    .gap_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.surface)
                    .child(
                        div().flex_1().child(Input::new(&self.input)),
                    )
                    .child(
                        Button::new("send_agent_msg")
                            .label(if self.sending { "Sending..." } else { "Send" })
                            .primary()
                            .disabled(self.sending)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.submit_message(window, cx);
                            })),
                    ),
            )
    }
}