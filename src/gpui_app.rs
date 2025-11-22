// Main GPUI Application
// Handles view routing and global state management

use gpui::*;
use gpui::prelude::*;
use gpui_component::button::{Button, ButtonVariants};
use crate::state::{AppState, ViewMode};
use crate::views::{chat_view::ChatView, settings_view::SettingsView, export_view::ExportView, organize_view::OrganizeView};

pub struct OverlookApp {
    state: Entity<AppState>,
}

impl OverlookApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        Self { state }
    }

    fn render_menu_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        div()
            .flex()
            .items_center()
            .gap_2()
            .p_2()
            .bg(state.theme.surface)
            .border_b_1()
            .border_color(state.theme.border)
            .child({
                let state_model = state_model.clone();
                Button::new("chat_nav")
                    .label("Chat")
                    .when(state.current_view == ViewMode::Chat, |button| {
                        button.primary()
                    })
                    .on_click(move |_, _, app| {
                        state_model.update(app, |state, _| {
                            state.switch_view(ViewMode::Chat);
                        });
                    })
            })
            .child({
                let state_model = state_model.clone();
                Button::new("settings_nav")
                    .label("Settings")
                    .when(state.current_view == ViewMode::Settings, |button| {
                        button.primary()
                    })
                    .on_click(move |_, _, app| {
                        state_model.update(app, |state, _| {
                            state.switch_view(ViewMode::Settings);
                        });
                    })
            })
            .child({
                let state_model = state_model.clone();
                Button::new("export_nav")
                    .label("Export")
                    .when(state.current_view == ViewMode::Export, |button| {
                        button.primary()
                    })
                    .on_click(move |_, _, app| {
                        state_model.update(app, |state, _| {
                            state.switch_view(ViewMode::Export);
                        });
                    })
            })
            .child({
                let state_model = state_model.clone();
                Button::new("organize_nav")
                    .label("Organize")
                    .when(state.current_view == ViewMode::Organize, |button| {
                        button.primary()
                    })
                    .on_click(move |_, _, app| {
                        state_model.update(app, |state, _| {
                            state.switch_view(ViewMode::Organize);
                        });
                    })
            })
    }

    fn render_current_view(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let current_view = self.state.read(cx).current_view;

        match current_view {
            ViewMode::Chat => {
                let view = cx.new(|_| ChatView::new(self.state.clone()));
                view.into_any_element()
            }
            ViewMode::Settings => {
                let view = cx.new(|_| SettingsView::new(self.state.clone()));
                view.into_any_element()
            }
            ViewMode::Export => {
                let view = cx.new(|_| ExportView::new(self.state.clone()));
                view.into_any_element()
            }
            ViewMode::Organize => {
                let view = cx.new(|_| OrganizeView::new(self.state.clone()));
                view.into_any_element()
            }
        }
    }
}

impl Render for OverlookApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(state.theme.background)
            .text_color(state.theme.text_primary)
            .child(self.render_menu_bar(cx))
            .child(
                div()
                    .flex_1()
                    .child(self.render_current_view(window, cx))
            )
    }
}
