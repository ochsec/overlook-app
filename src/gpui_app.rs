// Main GPUI Application
// Handles view routing and global state management

use gpui::*;
use gpui::prelude::*;
use gpui_component::button::{Button, ButtonVariants};
use crate::state::{AppState, ViewMode};
use crate::components::sidebar::Sidebar;
use crate::components::model_picker_modal::ModelPickerModal;
use crate::views::{chat_view::ChatView, settings_view::SettingsView, export_view::ExportView, organize_view::OrganizeView};

pub struct OverlookApp {
    state: Entity<AppState>,
    sidebar: Entity<Sidebar>,
    chat_view: Option<Entity<ChatView>>,
    chat_view_initialized: bool,
    model_picker: Option<Entity<ModelPickerModal>>,
    model_picker_initialized: bool,
}

impl OverlookApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        let sidebar = cx.new(|_| Sidebar::new(state.clone()));
        // Views will be created lazily when we have access to window
        Self {
            state,
            sidebar,
            chat_view: None,
            chat_view_initialized: false,
            model_picker: None,
            model_picker_initialized: false,
        }
    }

    fn ensure_chat_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.chat_view_initialized {
            let state = self.state.clone();
            self.chat_view = Some(cx.new(|cx| ChatView::new(state, window, cx)));
            self.chat_view_initialized = true;
        }
    }

    fn ensure_model_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model_picker_initialized {
            let state = self.state.clone();
            self.model_picker = Some(cx.new(|cx| ModelPickerModal::new(state, window, cx)));
            self.model_picker_initialized = true;
        }
    }

    fn render_menu_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(40.0))
            .px_4()
            .bg(state.theme.surface)
            .border_b_1()
            .border_color(state.theme.border)
            // Left side - View menu
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(state.theme.text_secondary)
                            .child("View:")
                    )
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_chat")
                            .label("Chat")
                            .compact();
                        if state.current_view == ViewMode::Chat {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::Chat);
                            });
                        })
                    })
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_settings")
                            .label("Settings")
                            .compact();
                        if state.current_view == ViewMode::Settings {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::Settings);
                            });
                        })
                    })
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_export")
                            .label("Export")
                            .compact();
                        if state.current_view == ViewMode::Export {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::Export);
                            });
                        })
                    })
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_organize")
                            .label("Organize")
                            .compact();
                        if state.current_view == ViewMode::Organize {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::Organize);
                            });
                        })
                    })
            )
            // Right side - App name
            .child(
                div()
                    .text_sm()
                    .text_color(state.theme.text_muted)
                    .child("Overlook")
            )
    }

    fn render_main_content_element(
        &self,
        current_view: ViewMode,
        chat_view: Option<Entity<ChatView>>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match current_view {
            ViewMode::Chat => {
                if let Some(chat_view) = chat_view {
                    chat_view.into_any_element()
                } else {
                    div().child("Chat view not initialized").into_any_element()
                }
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
        // Ensure views are initialized (need window)
        self.ensure_chat_view(window, cx);
        self.ensure_model_picker(window, cx);

        // Now read state and extract needed values
        let state = self.state.read(cx);
        let current_view = state.current_view;
        let theme_background = state.theme.background;
        let theme_text_primary = state.theme.text_primary;
        let show_model_picker = state.show_model_picker_modal;

        // Clone entities for use in element tree
        let sidebar = self.sidebar.clone();
        let chat_view = self.chat_view.clone();
        let model_picker = self.model_picker.clone();

        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme_background)
            .text_color(theme_text_primary)
            // Menu bar at top
            .child(self.render_menu_bar(cx))
            // Main area with sidebar + content
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    // Only show sidebar in Chat view
                    .when(current_view == ViewMode::Chat, |this| {
                        this.child(sidebar)
                    })
                    // Main content area
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .child(self.render_main_content_element(current_view, chat_view, cx))
                    )
            )
            // Model picker modal overlay
            .when(show_model_picker, |this| {
                if let Some(picker) = model_picker {
                    this.child(picker)
                } else {
                    this
                }
            })
    }
}
