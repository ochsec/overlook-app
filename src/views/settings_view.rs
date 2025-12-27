// Settings View - Application settings
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputState, Input};
use gpui_component::switch::Switch;
use gpui_component::slider::{SliderState, Slider, SliderEvent};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::{StyledExt, scroll::ScrollbarAxis};
use crate::gpui_theme::{GpuiTheme, ThemeType};
use crate::license::LicenseStatus;
use crate::state::AppState;

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
    // License key input
    license_input: Entity<InputState>,
}

impl SettingsView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (openrouter_key, openai_key, anthropic_key, lmstudio_url, ollama_url, font_size, license_key) = {
            let app_state = state.read(cx);
            (
                app_state.api_keys.openrouter.clone(),
                app_state.api_keys.openai.clone(),
                app_state.api_keys.anthropic.clone(),
                app_state.api_keys.lmstudio_url.clone(),
                app_state.api_keys.ollama_url.clone(),
                app_state.font_size,
                app_state.license_data.license_key.clone().unwrap_or_default(),
            )
        };

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

        let font_slider = cx.new(|_cx| {
            SliderState::new()
                .default_value(font_size)
                .min(10.0)
                .max(20.0)
                .step(1.0)
        });

        let license_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("XXXX-XXXX-XXXX-XXXX")
                .default_value(license_key)
        });

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
            license_input,
        }
    }

    // ============ Reusable Layout Helpers ============

    fn section_title(title: &str, theme: &GpuiTheme, font_size: f32) -> Div {
        div()
            .text_size(px(font_size + 4.0))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text_primary)
            .child(title.to_string())
    }

    fn section_container(theme: &GpuiTheme) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
    }

    fn labeled_row(label: &str, theme: &GpuiTheme, font_size: f32) -> Div {
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(
                div()
                    .w(px(100.0))
                    .text_size(px(font_size - 2.0))
                    .text_color(theme.text_secondary)
                    .child(label.to_string())
            )
    }

    fn toggle_row() -> Div {
        div().flex().items_center().gap_4()
    }

    // ============ Section Renderers ============

    fn render_header(&self, theme: &GpuiTheme, font_size: f32) -> impl IntoElement {
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
    }

    fn render_api_key_input(
        &self,
        label: &str,
        input: &Entity<InputState>,
        is_url: bool,
        theme: &GpuiTheme,
        font_size: f32,
    ) -> impl IntoElement {
        let input_clone = input.clone();
        let hint = if is_url { "Local server URL" } else { "API key stored locally" };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_size(px(font_size - 2.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_primary)
                    .child(label.to_string())
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(font_size))
                    .child(
                        div().flex_1().child({
                            let input_widget = Input::new(&input_clone).cleanable(true);
                            if is_url { input_widget } else { input_widget.mask_toggle() }
                        })
                    )
            )
            .child(
                div()
                    .text_size(px(font_size - 4.0))
                    .text_color(theme.text_secondary)
                    .child(hint)
            )
    }

    fn render_api_keys_section(
        &self,
        theme: &GpuiTheme,
        font_size: f32,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Self::section_title("Provider API Keys", theme, font_size))
                    .child(
                        div()
                            .text_size(px(font_size - 2.0))
                            .text_color(theme.text_muted)
                            .child("Keys are stored locally on this device.")
                    )
            )
            .child(self.render_api_key_input("OpenRouter", &self.openrouter_input, false, theme, font_size))
            .child(self.render_api_key_input("OpenAI", &self.openai_input, false, theme, font_size))
            .child(self.render_api_key_input("Anthropic", &self.anthropic_input, false, theme, font_size))
            .child(self.render_api_key_input("LM Studio", &self.lmstudio_input, true, theme, font_size))
            .child(self.render_api_key_input("Ollama", &self.ollama_input, true, theme, font_size))
    }

    fn render_appearance_section(
        &self,
        state_model: Entity<AppState>,
        theme: &GpuiTheme,
        font_size: f32,
        current_theme_type: ThemeType,
    ) -> impl IntoElement {
        Self::section_container(theme)
            .child(Self::section_title("Appearance", theme, font_size))
            .child(self.render_theme_selector(state_model, theme, font_size, current_theme_type))
            .child(self.render_font_size_slider(theme, font_size))
    }

    fn render_theme_selector(
        &self,
        state_model: Entity<AppState>,
        theme: &GpuiTheme,
        font_size: f32,
        current_theme_type: ThemeType,
    ) -> impl IntoElement {
        Self::labeled_row("Theme:", theme, font_size)
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
                                        Theme::change(ThemeMode::Dark, None, app);
                                    }
                                }),
                        )
                    })
            })
    }

    fn render_font_size_slider(&self, theme: &GpuiTheme, font_size: f32) -> impl IntoElement {
        Self::labeled_row("Font Size:", theme, font_size)
            .child(div().w(px(200.0)).child(Slider::new(&self.font_slider)))
            .child(
                div()
                    .text_size(px(font_size - 2.0))
                    .text_color(theme.text_muted)
                    .child(format!("{:.0}pt", font_size))
            )
    }

    fn render_chat_behavior_section(
        &self,
        state_model: Entity<AppState>,
        theme: &GpuiTheme,
        font_size: f32,
        streaming_enabled: bool,
        confirm_delete_chat: bool,
    ) -> impl IntoElement {
        Self::section_container(theme)
            .child(Self::section_title("Chat Behavior", theme, font_size))
            .child(self.render_toggle("streaming_toggle", "Enable streaming responses", streaming_enabled, {
                let state_model = state_model.clone();
                move |checked, _window, app| {
                    state_model.update(app, |state, _| {
                        state.streaming_enabled = *checked;
                    });
                }
            }))
            .child(self.render_toggle("confirm_delete_toggle", "Confirm before deleting chats", confirm_delete_chat, {
                let state_model = state_model.clone();
                move |checked, _window, app| {
                    state_model.update(app, |state, _| {
                        state.confirm_delete_chat = *checked;
                        crate::storage::Storage::save_confirm_delete_chat(*checked).ok();
                    });
                }
            }))
    }

    fn render_toggle<F>(
        &self,
        id: &'static str,
        label: &'static str,
        checked: bool,
        on_click: F,
    ) -> impl IntoElement
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        Self::toggle_row().child(
            Switch::new(id)
                .checked(checked)
                .label(label)
                .on_click(on_click)
        )
    }

    fn render_license_section(
        &self,
        state_model: Entity<AppState>,
        theme: &GpuiTheme,
        font_size: f32,
        license_status: &LicenseStatus,
        trial_expires: Option<String>,
    ) -> impl IntoElement {
        let status_color = match license_status {
            LicenseStatus::Valid { .. } => gpui::rgb(0x22c55e),
            LicenseStatus::Trial { .. } => gpui::rgb(0xeab308),
            _ => gpui::rgb(0xef4444),
        };
        let status_text = match license_status {
            LicenseStatus::Valid { email } => format!("Licensed to {}", email),
            LicenseStatus::Trial { days_remaining } => format!("Trial - {} days remaining", days_remaining),
            LicenseStatus::TrialExpired => "Trial expired".to_string(),
            LicenseStatus::Invalid { reason } => format!("Invalid: {}", reason),
        };
        let license_label = match &trial_expires {
            Some(expires) => format!("License Key (trial expires: {})", expires),
            None => "License Key".to_string(),
        };

        Self::section_container(theme)
            .child(Self::section_title("License", theme, font_size))
            .child(self.render_license_status(theme, font_size, status_color, &status_text))
            .child(self.render_license_input(state_model, theme, font_size, &license_label))
            .child(
                div()
                    .text_size(px(font_size - 4.0))
                    .text_color(theme.text_muted)
                    .child("Enter your license key and click Activate. Restart to validate.")
            )
    }

    fn render_license_status(
        &self,
        theme: &GpuiTheme,
        font_size: f32,
        status_color: Rgba,
        status_text: &str,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .text_size(px(font_size - 2.0))
            .child(div().text_color(theme.text_secondary).child("Status:"))
            .child(div().text_color(status_color).child(status_text.to_string()))
    }

    fn render_license_input(
        &self,
        state_model: Entity<AppState>,
        theme: &GpuiTheme,
        font_size: f32,
        label: &str,
    ) -> impl IntoElement {
        let license_input = self.license_input.clone();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_size(px(font_size - 2.0))
                    .text_color(theme.text_secondary)
                    .child(label.to_string())
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().flex_1().child(Input::new(&self.license_input).cleanable(true)))
                    .child(
                        Button::new("activate_license")
                            .label("Activate")
                            .on_click(move |_, _, cx| {
                                let key = license_input.read(cx).value().to_string();
                                if !key.is_empty() {
                                    state_model.update(cx, |state, _| {
                                        state.license_data.license_key = Some(key);
                                        state.license_data.validation_result = None;
                                        state.license_key_input = String::new();
                                        if let Err(e) = crate::license::save_license_data(&state.license_data) {
                                            eprintln!("Failed to save license: {}", e);
                                        }
                                    });
                                    println!("License key saved. Restart to validate.");
                                }
                            })
                    )
                    .child(
                        Button::new("purchase_license")
                            .label("Purchase")
                            .on_click(|_, _, _| {
                                let _ = open::that("https://overlook.app/#purchase");
                            })
                    )
            )
    }

    fn render_save_button(&self, state_model: Entity<AppState>) -> impl IntoElement {
        let openrouter_input = self.openrouter_input.clone();
        let openai_input = self.openai_input.clone();
        let anthropic_input = self.anthropic_input.clone();
        let lmstudio_input = self.lmstudio_input.clone();
        let ollama_input = self.ollama_input.clone();

        div()
            .flex()
            .justify_end()
            .p_4()
            .child(
                Button::new("save_settings")
                    .label("Save Settings")
                    .primary()
                    .on_click(move |_, _, cx| {
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

                            if let Err(e) = crate::storage::Storage::save_api_keys(&state.api_keys) {
                                println!("Failed to save API keys: {}", e);
                            }
                        });

                        println!("Settings saved!");
                    })
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
        let license_status = state.license_status.clone();

        let trial_expires = state.license_data.first_launch.map(|first| {
            let expires = first + chrono::Duration::days(30);
            expires.format("%Y-%m-%d %H:%M").to_string()
        });

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.text_primary)
            .child(self.render_header(&theme, font_size))
            .child(
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
                            .child(self.render_api_keys_section(&theme, font_size))
                            .child(self.render_appearance_section(state_model.clone(), &theme, font_size, current_theme_type))
                            .child(self.render_chat_behavior_section(state_model.clone(), &theme, font_size, streaming_enabled, confirm_delete_chat))
                            .child(self.render_license_section(state_model.clone(), &theme, font_size, &license_status, trial_expires))
                            .child(self.render_save_button(state_model))
                    )
            )
    }
}
