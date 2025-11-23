// Chat Input Component - Message input with attachments
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputState, Input};
use crate::state::AppState;

pub struct ChatInput {
    state: Entity<AppState>,
    input: Entity<InputState>,
}

impl ChatInput {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type your message here...")
        });
        Self { state, input }
    }
}

impl Render for ChatInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        let is_sending = state.sending_message || (state.streaming_active && !state.stream_completed);
        let send_label = if state.streaming_active && !state.stream_completed {
            "Streaming..."
        } else if state.sending_message {
            "Sending..."
        } else {
            "Send"
        };

        // Extract theme values
        let theme_surface = state.theme.surface;
        let theme_border = state.theme.border;
        let theme_text_primary = state.theme.text_primary;
        let theme_button_bg = state.theme.button_bg;

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
            .p_3()
            .gap_2()
            .bg(theme_surface)
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
                                    .px_2()
                                    .py_1()
                                    .bg(theme_button_bg)
                                    .rounded_md()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme_text_primary)
                                            .child(file_name.clone())
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("remove_attach_{}", idx)))
                                            .label("x")
                                            .compact()
                                            .on_click(move |_, _, app| {
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
            // Input row
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child({
                        // Attach button
                        let state_model = state_model.clone();
                        Button::new("attach_btn")
                            .label("Attach")
                            .on_click(move |_, _, app| {
                                state_model.update(app, |_state, _| {
                                    println!("Attach clicked");
                                });
                            })
                    })
                    .child(
                        // Text input area
                        div()
                            .flex_1()
                            .child(Input::new(&self.input).cleanable(true))
                    )
                    .child({
                        // Send button
                        let state_model = state_model.clone();
                        let mut btn = Button::new("send_btn")
                            .label(send_label)
                            .primary();

                        if !is_sending {
                            btn = btn.on_click(move |_, _, app| {
                                state_model.update(app, |_state, _| {
                                    println!("Send clicked");
                                });
                            });
                        }
                        btn
                    })
            )
    }
}
