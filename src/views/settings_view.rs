// Settings View - Application settings
use gpui::*;
use gpui::prelude::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputState, Input};
use gpui_component::switch::Switch;
use gpui_component::slider::{SliderState, Slider, SliderEvent};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use crate::state::AppState;
use crate::types::ProviderKind;
use crate::gpui_theme::ThemeType;

pub struct SettingsView {
    state: Entity<AppState>,
    // Input states for API keys
    openrouter_input: Entity<InputState>,
    openai_input: Entity<InputState>,
    anthropic_input: Entity<InputState>,
    lmstudio_input: Entity<InputState>,
    ollama_input: Entity<InputState>,
    // Slider state for font size
    font_slider: Entity<SliderState>,
}

impl SettingsView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Extract values from state first to avoid borrow issues
        let (openrouter_key, openai_key, anthropic_key, lmstudio_url, ollama_url, font_size) = {
            let app_state = state.read(cx);
            (
                app_state.api_keys.openrouter.clone(),
                app_state.api_keys.openai.clone(),
                app_state.api_keys.anthropic.clone(),
                app_state.api_keys.lmstudio_url.clone(),
                app_state.api_keys.ollama_url.clone(),
                app_state.font_size,
            )
        };

        // Create input states with initial values
        let openrouter_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter OpenRouter API key")
                .default_value(openrouter_key)
                .masked(true)
        });
        let openai_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter OpenAI API key")
                .default_value(openai_key)
                .masked(true)
        });
        let anthropic_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Enter Anthropic API key")
                .default_value(anthropic_key)
                .masked(true)
        });
        let lmstudio_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("http://localhost:1234")
                .default_value(lmstudio_url)
        });
        let ollama_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("http://localhost:11434")
                .default_value(ollama_url)
        });

        // Create font slider state
        let font_slider = cx.new(|_cx| {
            SliderState::new()
                .default_value(font_size)
                .min(10.0)
                .max(20.0)
                .step(1.0)
        });

        // Subscribe to slider changes
        let state_clone = state.clone();
        cx.subscribe(&font_slider, move |_this, _slider, event: &SliderEvent, cx| {
            if let SliderEvent::Change(value) = event {
                let size = match value {
                    gpui_component::slider::SliderValue::Single(v) => *v,
                    gpui_component::slider::SliderValue::Range(v, _) => *v,
                };
                state_clone.update(cx, |state, _| {
                    state.set_font_size(size);
                });
            }
        }).detach();

        Self {
            state,
            openrouter_input,
            openai_input,
            anthropic_input,
            lmstudio_input,
            ollama_input,
            font_slider,
        }
    }

    fn render_api_key_section(
        &self,
        _provider: ProviderKind,
        label: &str,
        input: &Entity<InputState>,
        is_url: bool,
        _state_model: Entity<AppState>,
        theme: &crate::gpui_theme::GpuiTheme,
        font_size: f32,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme_border = theme.border;
        let theme_text_primary = theme.text_primary;
        let theme_text_secondary = theme.text_secondary;
        let input_clone = input.clone();
        let label_owned = label.to_string();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(theme_border)
            .child(
                div()
                    .text_size(px(font_size - 2.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme_text_primary)
                    .child(label_owned)
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(font_size))
                    .child(
                        div()
                            .flex_1()
                            .child({
                                let input_widget = Input::new(&input_clone)
                                    .cleanable(true);
                                // Add mask toggle for API key inputs (not URLs)
                                if is_url {
                                    input_widget
                                } else {
                                    input_widget.mask_toggle()
                                }
                            })
                    )
            )
            .child(
                div()
                    .text_size(px(font_size - 4.0))
                    .text_color(theme_text_secondary)
                    .child(if is_url { "Local server URL" } else { "API key stored locally" })
            )
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let state_model = self.state.clone();

        let theme = state.theme.clone();
        let current_theme_type = state.theme_type;
        let streaming_enabled = state.streaming_enabled;
        let confirm_delete_chat = state.confirm_delete_chat;
        let font_size = state.font_size;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.text_primary)
            .child(
                // Header
                div()
                    .p_4()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_size(px(font_size + 6.0))
                            .font_weight(FontWeight::BOLD)
                            .child("Settings")
                    )
            )
            .child(
                // Scrollable content
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_6()
                            .p_4()
                            .scrollable(ScrollbarAxis::Vertical)
                            // API Keys Section
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(px(font_size + 4.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme.text_primary)
                                                    .child("Provider API Keys")
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(font_size - 2.0))
                                                    .text_color(theme.text_muted)
                                                    .child("Keys are stored locally on this device.")
                                            )
                                    )
                                    .child(self.render_api_key_section(
                                        ProviderKind::OpenRouter,
                                        "OpenRouter",
                                        &self.openrouter_input,
                                        false,
                                        state_model.clone(),
                                        &theme,
                                        font_size,
                                        cx,
                                    ))
                                    .child(self.render_api_key_section(
                                        ProviderKind::OpenAI,
                                        "OpenAI",
                                        &self.openai_input,
                                        false,
                                        state_model.clone(),
                                        &theme,
                                        font_size,
                                        cx,
                                    ))
                                    .child(self.render_api_key_section(
                                        ProviderKind::Anthropic,
                                        "Anthropic",
                                        &self.anthropic_input,
                                        false,
                                        state_model.clone(),
                                        &theme,
                                        font_size,
                                        cx,
                                    ))
                                    .child(self.render_api_key_section(
                                        ProviderKind::LMStudio,
                                        "LM Studio",
                                        &self.lmstudio_input,
                                        true,
                                        state_model.clone(),
                                        &theme,
                                        font_size,
                                        cx,
                                    ))
                                    .child(self.render_api_key_section(
                                        ProviderKind::Ollama,
                                        "Ollama",
                                        &self.ollama_input,
                                        true,
                                        state_model.clone(),
                                        &theme,
                                        font_size,
                                        cx,
                                    ))
                            )
                            // Appearance Section
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .p_4()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(theme.border)
                                    .child(
                                        div()
                                            .text_size(px(font_size + 4.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary)
                                            .child("Appearance")
                                    )
                                    // Theme selector
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_4()
                                            .child(
                                                div()
                                                    .w(px(100.0))
                                                    .text_size(px(font_size - 2.0))
                                                    .text_color(theme.text_secondary)
                                                    .child("Theme:")
                                            )
                                            .child({
                                                let state_model = state_model.clone();
                                                Button::new("theme_dropdown")
                                                    .label(SharedString::from(current_theme_type.name()))
                                                    .dropdown_menu(move |menu, _window, _cx| {
                                                        let state_model = state_model.clone();
                                                        menu.item(
                                                            PopupMenuItem::new("One Light")
                                                                .checked(current_theme_type == ThemeType::OneLight)
                                                                .on_click({
                                                                    let state_model = state_model.clone();
                                                                    move |_, _, app| {
                                                                        state_model.update(app, |state, _| {
                                                                            state.switch_theme(ThemeType::OneLight);
                                                                        });
                                                                        // Also update gpui_component theme
                                                                        Theme::change(ThemeMode::Light, None, app);
                                                                    }
                                                                }),
                                                        )
                                                        .item(
                                                            PopupMenuItem::new("One Dark")
                                                                .checked(current_theme_type == ThemeType::OneDark)
                                                                .on_click({
                                                                    let state_model = state_model.clone();
                                                                    move |_, _, app| {
                                                                        state_model.update(app, |state, _| {
                                                                            state.switch_theme(ThemeType::OneDark);
                                                                        });
                                                                        // Also update gpui_component theme
                                                                        Theme::change(ThemeMode::Dark, None, app);
                                                                    }
                                                                }),
                                                        )
                                                    })
                                            })
                                    )
                                    // Font size slider
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_4()
                                            .child(
                                                div()
                                                    .w(px(100.0))
                                                    .text_size(px(font_size - 2.0))
                                                    .text_color(theme.text_secondary)
                                                    .child("Font Size:")
                                            )
                                            .child(
                                                div()
                                                    .w(px(200.0))
                                                    .child(Slider::new(&self.font_slider))
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(font_size - 2.0))
                                                    .text_color(theme.text_muted)
                                                    .child(format!("{:.0}pt", font_size))
                                            )
                                    )
                            )
                            // Chat Behavior Section
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .p_4()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(theme.border)
                                    .child(
                                        div()
                                            .text_size(px(font_size + 4.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.text_primary)
                                            .child("Chat Behavior")
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_4()
                                            .child({
                                                let state_model = state_model.clone();
                                                Switch::new("streaming_toggle")
                                                    .checked(streaming_enabled)
                                                    .label("Enable streaming responses")
                                                    .on_click(move |checked, _, app| {
                                                        state_model.update(app, |state, _| {
                                                            state.streaming_enabled = *checked;
                                                        });
                                                    })
                                            })
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_4()
                                            .child({
                                                let state_model = state_model.clone();
                                                Switch::new("confirm_delete_toggle")
                                                    .checked(confirm_delete_chat)
                                                    .label("Confirm before deleting chats")
                                                    .on_click(move |checked, _, app| {
                                                        state_model.update(app, |state, _| {
                                                            state.confirm_delete_chat = *checked;
                                                            crate::storage::Storage::save_confirm_delete_chat(*checked).ok();
                                                        });
                                                    })
                                            })
                                    )
                            )
                            // Save Button
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .p_4()
                                    .child({
                                        let state_model = state_model.clone();
                                        let openrouter_input = self.openrouter_input.clone();
                                        let openai_input = self.openai_input.clone();
                                        let anthropic_input = self.anthropic_input.clone();
                                        let lmstudio_input = self.lmstudio_input.clone();
                                        let ollama_input = self.ollama_input.clone();

                                        Button::new("save_settings")
                                            .label("Save Settings")
                                            .primary()
                                            .on_click(move |_, _, cx| {
                                                // Read values from inputs and save
                                                let openrouter = openrouter_input.read(cx).value().to_string();
                                                let openai = openai_input.read(cx).value().to_string();
                                                let anthropic = anthropic_input.read(cx).value().to_string();
                                                let lmstudio = lmstudio_input.read(cx).value().to_string();
                                                let ollama = ollama_input.read(cx).value().to_string();

                                                state_model.update(cx, |state, _| {
                                                    state.api_keys.openrouter = openrouter;
                                                    state.api_keys.openai = openai;
                                                    state.api_keys.anthropic = anthropic;
                                                    state.api_keys.lmstudio_url = lmstudio;
                                                    state.api_keys.ollama_url = ollama;

                                                    // Save to storage
                                                    if let Err(e) = crate::storage::Storage::save_api_keys(&state.api_keys) {
                                                        println!("Failed to save API keys: {}", e);
                                                    }
                                                });

                                                println!("Settings saved!");
                                            })
                                    })
                            )
                    )
            )
    }
}
