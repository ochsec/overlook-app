// Main GPUI Application
// Handles view routing and global state management

use crate::api::{run_on_tokio_async, ApiClient};
use crate::components::agent_sidebar::AgentSidebar;
use crate::components::license_modal::LicenseModal;
use crate::components::model_picker_modal::ModelPickerModal;
use crate::components::sidebar::Sidebar;
use crate::state::{AppState, ViewMode};
use crate::views::{
    agent_chat::AgentChatView, chat_view::ChatView, export_view::ExportView,
    mcp_settings_view::McpSettingsView, organize_view::OrganizeView, settings_view::SettingsView,
};
use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};

pub struct OverlookApp {
    state: Entity<AppState>,
    sidebar: Entity<Sidebar>,
    agent_sidebar: Entity<AgentSidebar>,
    chat_view: Option<Entity<ChatView>>,
    chat_view_initialized: bool,
    agent_chat_view: Option<Entity<AgentChatView>>,
    agent_chat_view_initialized: bool,
    settings_view: Option<Entity<SettingsView>>,
    settings_view_initialized: bool,
    organize_view: Option<Entity<OrganizeView>>,
    organize_view_initialized: bool,
    model_picker: Option<Entity<ModelPickerModal>>,
    model_picker_initialized: bool,
    license_modal: Option<Entity<LicenseModal>>,
    license_modal_initialized: bool,
    mcp_settings_view: Option<Entity<McpSettingsView>>,
    mcp_settings_view_initialized: bool,
    models_load_started: bool,
    license_validation_started: bool,
}

impl OverlookApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.new(|_| AppState::new());
        let sidebar = cx.new(|_| Sidebar::new(state.clone()));
        let agent_sidebar = cx.new(|_| AgentSidebar::new(state.clone()));
        // Views will be created lazily when we have access to window
        Self {
            state,
            sidebar,
            agent_sidebar,
            chat_view: None,
            chat_view_initialized: false,
            agent_chat_view: None,
            agent_chat_view_initialized: false,
            settings_view: None,
            settings_view_initialized: false,
            organize_view: None,
            organize_view_initialized: false,
            model_picker: None,
            model_picker_initialized: false,
            license_modal: None,
            license_modal_initialized: false,
            mcp_settings_view: None,
            mcp_settings_view_initialized: false,
            models_load_started: false,
            license_validation_started: false,
        }
    }

    fn load_models_on_startup(&mut self, cx: &mut Context<Self>) {
        if self.models_load_started {
            return;
        }

        self.models_load_started = true;

        // Auto-connect MCP servers on startup
        self.state.read(cx).connect_mcp_servers(&mut cx.to_async());

        // Get provider and API key
        let (provider, api_key, has_key) = {
            let state = self.state.read(cx);
            let key = state.api_keys.get(state.current_provider).to_string();
            (state.current_provider, key.clone(), !key.trim().is_empty())
        };

        // Only load models if we have an API key
        if !has_key {
            return;
        }

        // Set loading state
        self.state.update(cx, |state, _| {
            state.loading_models = true;
        });

        // Spawn async task to load models
        let state_model = self.state.clone();
        cx.spawn({
            let state_model = state_model.clone();
            async move |_this, cx| {
                let result = run_on_tokio_async(async move {
                    let api_client = ApiClient::new();
                    api_client.load_models(provider, &api_key).await
                })
                .await;

                cx.update(|cx| {
                    state_model.update(cx, |state, _| {
                        state.loading_models = false;
                        match result {
                            Ok(models) => {
                                println!("Loaded {} models on startup", models.len());
                                state.available_models = models;
                                // Auto-select first model if none selected
                                if state.current_model.is_empty()
                                    && !state.available_models.is_empty()
                                {
                                    state.current_model = state.available_models[0].id.clone();
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to load models on startup: {}", e);
                            }
                        }
                    });
                })
                .ok();
            }
        })
        .detach();
    }

    fn validate_license_on_startup(&mut self, cx: &mut Context<Self>) {
        if self.license_validation_started {
            return;
        }

        let needs_validation = {
            let state = self.state.read(cx);
            state.license_data.license_key.is_some()
                && state.license_data.validation_result.is_none()
        };

        if !needs_validation {
            self.license_validation_started = true;
            return;
        }

        self.license_validation_started = true;

        let (license_key, hardware_id) = {
            let state = self.state.read(cx);
            (
                state.license_data.license_key.clone().unwrap_or_default(),
                state
                    .license_data
                    .hardware_id
                    .clone()
                    .unwrap_or_else(crate::license::generate_fingerprint),
            )
        };

        let state_model = self.state.clone();
        cx.spawn({
            let state_model = state_model.clone();
            async move |_this, cx| {
                let result = run_on_tokio_async({
                    let license_key = license_key.clone();
                    let hardware_id = hardware_id.clone();
                    async move {
                        let client = crate::license::LicenseClient::new();
                        client.validate(&license_key, &hardware_id).await
                    }
                })
                .await;

                cx.update(|cx| {
                    state_model.update(cx, |state, _| match result {
                        Ok(validation_result) => {
                            state.license_data.last_validation = Some(chrono::Utc::now());
                            state.license_data.validation_result = Some(validation_result.clone());

                            if validation_result.valid {
                                state.license_status = crate::license::LicenseStatus::Valid {
                                    email: validation_result.email.unwrap_or_default(),
                                };
                                state.show_license_modal = false;
                            } else {
                                state.license_status = crate::license::LicenseStatus::Invalid {
                                    reason: validation_result
                                        .error
                                        .unwrap_or_else(|| "Invalid license".to_string()),
                                };
                            }

                            if let Err(e) = crate::license::save_license_data(&state.license_data) {
                                eprintln!("Failed to save license data: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("License validation failed: {}", e);
                        }
                    });
                })
                .ok();

                Ok::<_, anyhow::Error>(())
            }
        })
        .detach();
    }

    fn ensure_chat_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.chat_view_initialized {
            let state = self.state.clone();
            self.chat_view = Some(cx.new(|cx| ChatView::new(state, window, cx)));
            self.chat_view_initialized = true;
        }
    }

    fn ensure_settings_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.settings_view_initialized {
            let state = self.state.clone();
            self.settings_view = Some(cx.new(|cx| SettingsView::new(state, window, cx)));
            self.settings_view_initialized = true;
        }
    }

    fn ensure_organize_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.organize_view_initialized {
            let state = self.state.clone();
            self.organize_view = Some(cx.new(|cx| OrganizeView::new(state, window, cx)));
            self.organize_view_initialized = true;
        }
    }

    fn ensure_model_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model_picker_initialized {
            let state = self.state.clone();
            self.model_picker = Some(cx.new(|cx| ModelPickerModal::new(state, window, cx)));
            self.model_picker_initialized = true;
        }
    }

    fn ensure_license_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.license_modal_initialized {
            let state = self.state.clone();
            self.license_modal = Some(cx.new(|cx| LicenseModal::new(state, window, cx)));
            self.license_modal_initialized = true;
        }
    }

    fn ensure_mcp_settings_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.mcp_settings_view_initialized {
            let state = self.state.clone();
            self.mcp_settings_view = Some(cx.new(|cx| McpSettingsView::new(state, window, cx)));
            self.mcp_settings_view_initialized = true;
        }
    }

    fn ensure_agent_chat_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.agent_chat_view_initialized {
            let state = self.state.clone();
            self.agent_chat_view = Some(cx.new(|cx| AgentChatView::new(state, window, cx)));
            self.agent_chat_view_initialized = true;
        }
    }

    fn render_menu_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();
        let font_size = state.font_size;

        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(40.0))
            .px_4()
            .bg(state.theme.surface)
            .border_b_1()
            .border_color(state.theme.border)
            .text_size(px(font_size))
            // Left side - View menu
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(font_size - 2.0))
                            .text_color(state.theme.text_secondary)
                            .child("View:"),
                    )
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_chat").label("Chat").compact();
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
                        let mut btn = Button::new("nav_agent").label("Agent").compact();
                        if state.current_view == ViewMode::AgentChat {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::AgentChat);
                            });
                        })
                    })
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_settings").label("Settings").compact();
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
                        let mut btn = Button::new("nav_export").label("Export").compact();
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
                        let mut btn = Button::new("nav_organize").label("Organize").compact();
                        if state.current_view == ViewMode::Organize {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::Organize);
                            });
                        })
                    })
                    .child({
                        let state_model = state_model.clone();
                        let mut btn = Button::new("nav_mcp").label("MCP").compact();
                        if state.current_view == ViewMode::McpSettings {
                            btn = btn.primary();
                        }
                        btn.on_click(move |_, _, app| {
                            state_model.update(app, |state, _| {
                                state.switch_view(ViewMode::McpSettings);
                            });
                        })
                    }),
            )
            // Right side - App name
            .child(
                div()
                    .text_size(px(font_size - 2.0))
                    .text_color(state.theme.text_muted)
                    .child("Overlook"),
            )
    }

    fn render_main_content_element(
        &self,
        current_view: ViewMode,
        chat_view: Option<Entity<ChatView>>,
        agent_chat_view: Option<Entity<AgentChatView>>,
        settings_view: Option<Entity<SettingsView>>,
        organize_view: Option<Entity<OrganizeView>>,
        mcp_settings_view: Option<Entity<McpSettingsView>>,
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
            ViewMode::AgentChat => {
                if let Some(agent_chat_view) = agent_chat_view {
                    agent_chat_view.into_any_element()
                } else {
                    div()
                        .child("Agent Chat view not initialized")
                        .into_any_element()
                }
            }
            ViewMode::Settings => {
                if let Some(settings_view) = settings_view {
                    settings_view.into_any_element()
                } else {
                    div()
                        .child("Settings view not initialized")
                        .into_any_element()
                }
            }
            ViewMode::Export => {
                let view = cx.new(|_| ExportView::new(self.state.clone()));
                view.into_any_element()
            }
            ViewMode::Organize => {
                if let Some(organize_view) = organize_view {
                    organize_view.into_any_element()
                } else {
                    div()
                        .child("Organize view not initialized")
                        .into_any_element()
                }
            }
            ViewMode::McpSettings => {
                if let Some(mcp_settings_view) = mcp_settings_view {
                    mcp_settings_view.into_any_element()
                } else {
                    div()
                        .child("MCP Settings view not initialized")
                        .into_any_element()
                }
            }
        }
    }
}

impl Render for OverlookApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Ensure views are initialized (need window)
        self.ensure_chat_view(window, cx);
        self.ensure_agent_chat_view(window, cx);
        self.ensure_settings_view(window, cx);
        self.ensure_organize_view(window, cx);
        self.ensure_mcp_settings_view(window, cx);
        self.ensure_model_picker(window, cx);
        self.ensure_license_modal(window, cx);

        // Auto-load models on startup if API key is configured
        self.load_models_on_startup(cx);

        // Validate license on startup if key present but not validated
        self.validate_license_on_startup(cx);

        // Now read state and extract needed values
        let state = self.state.read(cx);
        let current_view = state.current_view;
        let theme_background = state.theme.background;
        let theme_text_primary = state.theme.text_primary;
        let show_model_picker = state.show_model_picker_modal;
        let show_license_modal = state.show_license_modal;

        // Clone entities for use in element tree
        let sidebar = self.sidebar.clone();
        let agent_sidebar = self.agent_sidebar.clone();
        let chat_view = self.chat_view.clone();
        let agent_chat_view = self.agent_chat_view.clone();
        let settings_view = self.settings_view.clone();
        let organize_view = self.organize_view.clone();
        let mcp_settings_view = self.mcp_settings_view.clone();
        let model_picker = self.model_picker.clone();
        let license_modal = self.license_modal.clone();

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
                    .when(current_view == ViewMode::Chat, |this| this.child(sidebar))
                    // Show agent sidebar in Agent Chat view
                    .when(current_view == ViewMode::AgentChat, |this| this.child(agent_sidebar))
                    // Main content area
                    .child(div().flex_1().overflow_hidden().child(
                        self.render_main_content_element(
                            current_view,
                            chat_view,
                            agent_chat_view,
                            settings_view,
                            organize_view,
                            mcp_settings_view,
                            cx,
                        ),
                    )),
            )
            // Model picker modal overlay
            .when(show_model_picker, |this| {
                if let Some(picker) = model_picker {
                    this.child(picker)
                } else {
                    this
                }
            })
            // License expired modal overlay (takes priority)
            .when(show_license_modal, |this| {
                if let Some(modal) = license_modal {
                    this.child(modal)
                } else {
                    this
                }
            })
    }
}
