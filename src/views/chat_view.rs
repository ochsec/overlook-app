// Chat View - Main chat interface
use gpui::*;
use gpui::prelude::*;
use crate::state::AppState;

pub struct ChatView {
    state: Entity<AppState>,
}

impl ChatView {
    pub fn new(state: Entity<AppState>) -> Self {
        Self { state }
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
            .child(
                div()
                    .p_4()
                    .child("Chat View - Coming in Phase 4")
            )
    }
}
