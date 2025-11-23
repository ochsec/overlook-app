// Chat Header Component - Provider and Model selection
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::Button;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use crate::state::AppState;
use crate::types::ProviderKind;

pub struct ChatHeader {
    state: Entity<AppState>,
}

impl ChatHeader {
    pub fn new(state: Entity<AppState>) -> Self {
        Self { state }
    }
}

impl Render for ChatHeader {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        let provider_has_key = !state.api_keys.get(state.current_provider).trim().is_empty();
        let current_provider = state.current_provider;
        let theme_surface = state.theme.surface;
        let theme_border = state.theme.border;
        let theme_text_secondary = state.theme.text_secondary;
        let theme_text_muted = state.theme.text_muted;

        let model_label = if state.loading_models {
            "Loading models...".to_string()
        } else if !provider_has_key {
            "Enter API key first".to_string()
        } else if state.available_models.is_empty() {
            "No models loaded".to_string()
        } else if state.current_model.is_empty() {
            "Select a model...".to_string()
        } else {
            state.available_models
                .iter()
                .find(|m| m.id == state.current_model)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| state.current_model.clone())
        };

        div()
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .bg(theme_surface)
            .border_b_1()
            .border_color(theme_border)
            .child(
                // Provider label
                div()
                    .text_sm()
                    .text_color(theme_text_secondary)
                    .child("Provider:")
            )
            .child({
                // Provider dropdown
                let state_model_clone = state_model.clone();
                Button::new("provider_dropdown")
                    .label(SharedString::from(current_provider.display_name()))
                    .dropdown_menu(move |menu, _window, _cx| {
                        let state_model = state_model_clone.clone();
                        menu.item(
                            PopupMenuItem::new("OpenRouter")
                                .checked(current_provider == ProviderKind::OpenRouter)
                                .on_click({
                                    let state_model = state_model.clone();
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.current_provider = ProviderKind::OpenRouter;
                                            state.current_model.clear();
                                        });
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("OpenAI")
                                .checked(current_provider == ProviderKind::OpenAI)
                                .on_click({
                                    let state_model = state_model.clone();
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.current_provider = ProviderKind::OpenAI;
                                            state.current_model.clear();
                                        });
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("Anthropic")
                                .checked(current_provider == ProviderKind::Anthropic)
                                .on_click({
                                    let state_model = state_model.clone();
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.current_provider = ProviderKind::Anthropic;
                                            state.current_model.clear();
                                        });
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("LM Studio")
                                .checked(current_provider == ProviderKind::LMStudio)
                                .on_click({
                                    let state_model = state_model.clone();
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.current_provider = ProviderKind::LMStudio;
                                            state.current_model.clear();
                                        });
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("Ollama")
                                .checked(current_provider == ProviderKind::Ollama)
                                .on_click({
                                    let state_model = state_model.clone();
                                    move |_, _, app| {
                                        state_model.update(app, |state, _| {
                                            state.current_provider = ProviderKind::Ollama;
                                            state.current_model.clear();
                                        });
                                    }
                                }),
                        )
                    })
            })
            .child(
                // Spacer
                div().w(px(16.0))
            )
            .child(
                // Model label
                div()
                    .text_sm()
                    .text_color(theme_text_secondary)
                    .child("Model:")
            )
            .child({
                // Model picker button (opens modal)
                let state_model = state_model.clone();
                Button::new("model_picker")
                    .label(SharedString::from(format!("🔍 {}", model_label)))
                    .on_click(move |_, _, app| {
                        state_model.update(app, |state, _| {
                            state.show_model_picker_modal = true;
                            state.model_search_query.clear();
                        });
                    })
            })
            .when(!provider_has_key, |this| {
                this.child(
                    div()
                        .text_sm()
                        .italic()
                        .text_color(theme_text_muted)
                        .child("← add a key in Settings")
                )
            })
            .child({
                // Refresh button
                let state_model = state_model.clone();
                Button::new("refresh_models")
                    .label("🔄")
                    .on_click(move |_, _, app| {
                        state_model.update(app, |_state, _| {
                            println!("Refresh models clicked");
                        });
                    })
            })
    }
}
