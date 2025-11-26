// Chat Header Component - Provider and Model selection
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::Button;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use crate::api::{ApiClient, run_on_tokio_async};
use crate::state::AppState;
use crate::storage::Storage;
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

        let api_key = state.api_keys.get(state.current_provider).to_string();
        let provider_has_key = !api_key.trim().is_empty();
        let current_provider = state.current_provider;
        let theme_surface = state.theme.surface;
        let theme_border = state.theme.border;
        let theme_text_secondary = state.theme.text_secondary;
        let theme_text_muted = state.theme.text_muted;
        let font_size = state.font_size;

        // Capture API keys for each provider (for loading on provider change)
        let api_keys = state.api_keys.clone();

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
                    .text_size(px(font_size - 2.0))
                    .text_color(theme_text_secondary)
                    .child("Provider:")
            )
            .child({
                // Provider dropdown
                let state_model_clone = state_model.clone();
                let api_keys_clone = api_keys.clone();
                Button::new("provider_dropdown")
                    .label(SharedString::from(current_provider.display_name()))
                    .dropdown_menu(move |menu, _window, _cx| {
                        let state_model = state_model_clone.clone();
                        let api_keys = api_keys_clone.clone();
                        menu.item(
                            PopupMenuItem::new("OpenRouter")
                                .checked(current_provider == ProviderKind::OpenRouter)
                                .on_click({
                                    let state_model = state_model.clone();
                                    let api_key = api_keys.openrouter.clone();
                                    move |_, _, app| {
                                        let provider = ProviderKind::OpenRouter;
                                        let state_model = state_model.clone();
                                        let api_key = api_key.clone();

                                        state_model.update(app, |state, _| {
                                            state.current_provider = provider;
                                            state.current_model.clear();
                                            state.available_models.clear();
                                            state.loading_models = true;
                                            // Save the last used provider
                                            if let Err(e) = Storage::save_last_provider(&provider) {
                                                eprintln!("Failed to save last provider: {}", e);
                                            }
                                        });

                                        // Spawn async task to load models
                                        app.spawn({
                                            let state_model = state_model.clone();
                                            async move |cx| {
                                                let result = run_on_tokio_async(async move {
                                                    let api_client = ApiClient::new();
                                                    api_client.load_models(provider, &api_key).await
                                                }).await;
                                                cx.update(|cx| {
                                                    state_model.update(cx, |state, _| {
                                                        state.loading_models = false;
                                                        if let Ok(models) = result {
                                                            state.available_models = models;
                                                            if state.current_model.is_empty() && !state.available_models.is_empty() {
                                                                state.current_model = state.available_models[0].id.clone();
                                                            }
                                                        }
                                                    });
                                                }).ok();
                                            }
                                        }).detach();
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("OpenAI")
                                .checked(current_provider == ProviderKind::OpenAI)
                                .on_click({
                                    let state_model = state_model.clone();
                                    let api_key = api_keys.openai.clone();
                                    move |_, _, app| {
                                        let provider = ProviderKind::OpenAI;
                                        let state_model = state_model.clone();
                                        let api_key = api_key.clone();

                                        state_model.update(app, |state, _| {
                                            state.current_provider = provider;
                                            state.current_model.clear();
                                            state.available_models.clear();
                                            state.loading_models = true;
                                            // Save the last used provider
                                            if let Err(e) = Storage::save_last_provider(&provider) {
                                                eprintln!("Failed to save last provider: {}", e);
                                            }
                                        });

                                        app.spawn({
                                            let state_model = state_model.clone();
                                            async move |cx| {
                                                let result = run_on_tokio_async(async move {
                                                    let api_client = ApiClient::new();
                                                    api_client.load_models(provider, &api_key).await
                                                }).await;
                                                cx.update(|cx| {
                                                    state_model.update(cx, |state, _| {
                                                        state.loading_models = false;
                                                        if let Ok(models) = result {
                                                            state.available_models = models;
                                                            if state.current_model.is_empty() && !state.available_models.is_empty() {
                                                                state.current_model = state.available_models[0].id.clone();
                                                            }
                                                        }
                                                    });
                                                }).ok();
                                            }
                                        }).detach();
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("Anthropic")
                                .checked(current_provider == ProviderKind::Anthropic)
                                .on_click({
                                    let state_model = state_model.clone();
                                    let api_key = api_keys.anthropic.clone();
                                    move |_, _, app| {
                                        let provider = ProviderKind::Anthropic;
                                        let state_model = state_model.clone();
                                        let api_key = api_key.clone();

                                        state_model.update(app, |state, _| {
                                            state.current_provider = provider;
                                            state.current_model.clear();
                                            state.available_models.clear();
                                            state.loading_models = true;
                                            // Save the last used provider
                                            if let Err(e) = Storage::save_last_provider(&provider) {
                                                eprintln!("Failed to save last provider: {}", e);
                                            }
                                        });

                                        app.spawn({
                                            let state_model = state_model.clone();
                                            async move |cx| {
                                                let result = run_on_tokio_async(async move {
                                                    let api_client = ApiClient::new();
                                                    api_client.load_models(provider, &api_key).await
                                                }).await;
                                                cx.update(|cx| {
                                                    state_model.update(cx, |state, _| {
                                                        state.loading_models = false;
                                                        if let Ok(models) = result {
                                                            state.available_models = models;
                                                            if state.current_model.is_empty() && !state.available_models.is_empty() {
                                                                state.current_model = state.available_models[0].id.clone();
                                                            }
                                                        }
                                                    });
                                                }).ok();
                                            }
                                        }).detach();
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("LM Studio")
                                .checked(current_provider == ProviderKind::LMStudio)
                                .on_click({
                                    let state_model = state_model.clone();
                                    let api_key = api_keys.lmstudio_url.clone();
                                    move |_, _, app| {
                                        let provider = ProviderKind::LMStudio;
                                        let state_model = state_model.clone();
                                        let api_key = api_key.clone();

                                        state_model.update(app, |state, _| {
                                            state.current_provider = provider;
                                            state.current_model.clear();
                                            state.available_models.clear();
                                            state.loading_models = true;
                                            // Save the last used provider
                                            if let Err(e) = Storage::save_last_provider(&provider) {
                                                eprintln!("Failed to save last provider: {}", e);
                                            }
                                        });

                                        app.spawn({
                                            let state_model = state_model.clone();
                                            async move |cx| {
                                                let result = run_on_tokio_async(async move {
                                                    let api_client = ApiClient::new();
                                                    api_client.load_models(provider, &api_key).await
                                                }).await;
                                                cx.update(|cx| {
                                                    state_model.update(cx, |state, _| {
                                                        state.loading_models = false;
                                                        if let Ok(models) = result {
                                                            state.available_models = models;
                                                            if state.current_model.is_empty() && !state.available_models.is_empty() {
                                                                state.current_model = state.available_models[0].id.clone();
                                                            }
                                                        }
                                                    });
                                                }).ok();
                                            }
                                        }).detach();
                                    }
                                }),
                        )
                        .item(
                            PopupMenuItem::new("Ollama")
                                .checked(current_provider == ProviderKind::Ollama)
                                .on_click({
                                    let state_model = state_model.clone();
                                    let api_key = api_keys.ollama_url.clone();
                                    move |_, _, app| {
                                        let provider = ProviderKind::Ollama;
                                        let state_model = state_model.clone();
                                        let api_key = api_key.clone();

                                        state_model.update(app, |state, _| {
                                            state.current_provider = provider;
                                            state.current_model.clear();
                                            state.available_models.clear();
                                            state.loading_models = true;
                                            // Save the last used provider
                                            if let Err(e) = Storage::save_last_provider(&provider) {
                                                eprintln!("Failed to save last provider: {}", e);
                                            }
                                        });

                                        app.spawn({
                                            let state_model = state_model.clone();
                                            async move |cx| {
                                                let result = run_on_tokio_async(async move {
                                                    let api_client = ApiClient::new();
                                                    api_client.load_models(provider, &api_key).await
                                                }).await;
                                                cx.update(|cx| {
                                                    state_model.update(cx, |state, _| {
                                                        state.loading_models = false;
                                                        if let Ok(models) = result {
                                                            state.available_models = models;
                                                            if state.current_model.is_empty() && !state.available_models.is_empty() {
                                                                state.current_model = state.available_models[0].id.clone();
                                                            }
                                                        }
                                                    });
                                                }).ok();
                                            }
                                        }).detach();
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
                    .text_size(px(font_size - 2.0))
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
                        .text_size(px(font_size - 2.0))
                        .italic()
                        .text_color(theme_text_muted)
                        .child("← add a key in Settings")
                )
            })
            .child({
                // Refresh button
                let state_model = state_model.clone();
                let api_key = api_key.clone();
                Button::new("refresh_models")
                    .label("🔄")
                    .on_click(move |_, _, app| {
                        let state_model = state_model.clone();
                        let api_key = api_key.clone();

                        // Set loading state
                        state_model.update(app, |state, _| {
                            state.loading_models = true;
                            state.available_models.clear();
                        });

                        // Spawn async task to load models
                        app.spawn({
                            let state_model = state_model.clone();
                            async move |cx| {
                                // Run reqwest on Tokio runtime (GPUI uses smol)
                                let result = run_on_tokio_async(async move {
                                    let api_client = ApiClient::new();
                                    api_client.load_models(current_provider, &api_key).await
                                }).await;

                                cx.update(|cx| {
                                    state_model.update(cx, |state, _| {
                                        state.loading_models = false;
                                        match result {
                                            Ok(models) => {
                                                println!("Loaded {} models", models.len());
                                                state.available_models = models;
                                                // Auto-select first model if none selected
                                                if state.current_model.is_empty() && !state.available_models.is_empty() {
                                                    state.current_model = state.available_models[0].id.clone();
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to load models: {}", e);
                                            }
                                        }
                                    });
                                }).ok();
                            }
                        }).detach();
                    })
            })
    }
}
