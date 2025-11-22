// Settings View - Application settings
use gpui::*;
use gpui::prelude::*;
use crate::state::AppState;

pub struct SettingsView {
    state: Entity<AppState>,
}

impl SettingsView {
    pub fn new(state: Entity<AppState>) -> Self {
        Self { state }
    }
}

impl Render for SettingsView {
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
                    .child("Settings View - Coming in Phase 3")
            )
    }
}
