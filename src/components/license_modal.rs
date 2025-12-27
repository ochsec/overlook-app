// License Modal Component
// Shows when trial has expired and no valid license key is present

use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};

use crate::state::AppState;

pub struct LicenseModal {
    state: Entity<AppState>,
    license_input: Entity<InputState>,
}

impl LicenseModal {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let license_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("XXXX-XXXX-XXXX-XXXX")
        });

        Self {
            state,
            license_input,
        }
    }
}

impl Render for LicenseModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let theme = &state.theme;
        let font_size = state.font_size;

        // Calculate trial expiration date
        let trial_expired_at = state.license_data.first_launch.map(|first| {
            let expires = first + chrono::Duration::days(30);
            expires.format("%Y-%m-%d %H:%M").to_string()
        });

        // Full-screen overlay
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::rgba(0x000000aa)) // Semi-transparent black overlay
            .child(
                // Modal container
                div()
                    .w(px(450.0))
                    .p_6()
                    .rounded_lg()
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .gap_4()
                    // Title
                    .child(
                        div()
                            .text_size(px(font_size + 6.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .text_center()
                            .child("Trial Expired")
                    )
                    // Message
                    .child(
                        div()
                            .text_size(px(font_size))
                            .text_color(theme.text_secondary)
                            .text_center()
                            .child("Your 30-day trial has ended. Please enter a license key to continue using Overlook.")
                    )
                    // License key input
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(font_size - 2.0))
                                    .text_color(theme.text_secondary)
                                    .child(match &trial_expired_at {
                                        Some(expired) => format!("License Key (trial expired: {})", expired),
                                        None => "License Key".to_string(),
                                    })
                            )
                            .child(
                                Input::new(&self.license_input)
                                    .cleanable(true)
                            )
                    )
                    // Buttons
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_center()
                            .child({
                                let state_model = self.state.clone();
                                let license_input = self.license_input.clone();
                                Button::new("activate_license")
                                    .label("Activate License")
                                    .primary()
                                    .on_click(move |_, _, cx| {
                                        let key = license_input.read(cx).value().to_string();
                                        if !key.is_empty() {
                                            state_model.update(cx, |state, _| {
                                                state.license_data.license_key = Some(key);
                                                state.license_data.validation_result = None;
                                                state.show_license_modal = false;
                                                if let Err(e) = crate::license::save_license_data(&state.license_data) {
                                                    eprintln!("Failed to save license: {}", e);
                                                }
                                            });
                                            println!("License key saved. Restart to validate.");
                                        }
                                    })
                            })
                            .child({
                                Button::new("purchase_license")
                                    .label("Purchase License")
                                    .on_click(|_, _, _cx| {
                                        // Open purchase URL in browser
                                        if let Err(e) = open::that("https://overlook.app/#purchase") {
                                            eprintln!("Failed to open purchase URL: {}", e);
                                        }
                                    })
                            })
                    )
            )
    }
}
