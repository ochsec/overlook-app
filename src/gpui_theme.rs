// GPUI Theme System
// Ported from egui theme system to GPUI

use gpui::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeType {
    OneLight,
    OneDark,
}

impl Default for ThemeType {
    fn default() -> Self {
        ThemeType::OneDark
    }
}

impl ThemeType {
    pub fn name(&self) -> &'static str {
        match self {
            ThemeType::OneLight => "One Light",
            ThemeType::OneDark => "One Dark",
        }
    }

    pub fn all_themes() -> Vec<ThemeType> {
        vec![ThemeType::OneLight, ThemeType::OneDark]
    }
}

#[derive(Clone, Debug)]
pub struct GpuiTheme {
    // Background colors
    pub background: Hsla,
    pub surface: Hsla,
    pub sidebar_background: Hsla,

    // Text colors
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,

    // UI element colors
    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub border: Hsla,
    pub separator: Hsla,
    pub danger: Hsla,

    // Message bubble colors
    pub user_message_bg: Hsla,
    pub user_message_text: Hsla,
    pub assistant_message_bg: Hsla,
    pub assistant_message_text: Hsla,

    // Button colors
    pub button_bg: Hsla,
    pub button_bg_hover: Hsla,
    pub button_text: Hsla,
    pub button_primary_bg: Hsla,
    pub button_primary_bg_hover: Hsla,
    pub button_primary_text: Hsla,

    // Input colors
    pub input_bg: Hsla,
    pub input_border: Hsla,
    pub input_text: Hsla,
}

impl GpuiTheme {
    pub fn one_light() -> Self {
        GpuiTheme {
            // One Light - Based on Atom's One Light theme
            background: rgb(0xfafafa).into(),         // #fafafa
            surface: rgb(0xffffff).into(),            // #ffffff
            sidebar_background: rgb(0xf5f5f5).into(), // #f5f5f5

            text_primary: rgb(0x383a42).into(),   // #383a42
            text_secondary: rgb(0x6a737d).into(), // #6a737d
            text_muted: rgb(0xa0a1a7).into(),     // #a0a1a7

            accent: rgb(0x4078f2).into(),       // #4078f2
            accent_hover: rgb(0x2d64dc).into(), // #2d64dc
            border: rgb(0xe4e5e9).into(),       // #e4e5e9
            separator: rgb(0xeaecef).into(),    // #eaecef
            danger: rgb(0xe45649).into(),       // #e45649 (One Light red)

            user_message_bg: rgb(0x4078f2).into(), // #4078f2
            user_message_text: white(),
            assistant_message_bg: rgb(0xf0f0f0).into(), // #f0f0f0
            assistant_message_text: rgb(0x383a42).into(), // #383a42

            button_bg: rgb(0xffffff).into(),               // #ffffff
            button_bg_hover: rgb(0xf5f5f5).into(),         // #f5f5f5
            button_text: rgb(0x383a42).into(),             // #383a42
            button_primary_bg: rgb(0x4078f2).into(),       // #4078f2
            button_primary_bg_hover: rgb(0x2d64dc).into(), // #2d64dc
            button_primary_text: white(),

            input_bg: rgb(0xffffff).into(),     // #ffffff
            input_border: rgb(0xe4e5e9).into(), // #e4e5e9
            input_text: rgb(0x383a42).into(),   // #383a42
        }
    }

    pub fn one_dark() -> Self {
        GpuiTheme {
            // One Dark - Based on Atom's One Dark theme with improved contrast
            background: rgb(0x282c34).into(),         // #282c34
            surface: rgb(0x21252b).into(),            // #21252b
            sidebar_background: rgb(0x23272e).into(), // #23272e

            text_primary: rgb(0xdcdfe4).into(),   // #dcdfe4
            text_secondary: rgb(0xb4bbc5).into(), // #b4bbc5
            text_muted: rgb(0x828997).into(),     // #828997

            accent: rgb(0x61afef).into(),       // #61afef
            accent_hover: rgb(0x4b9bdc).into(), // #4b9bdc
            border: rgb(0x373b45).into(),       // #373b45
            separator: rgb(0x323640).into(),    // #323640
            danger: rgb(0xe06c75).into(),       // #e06c75 (One Dark red)

            user_message_bg: rgb(0x61afef).into(), // #61afef
            user_message_text: white(),
            assistant_message_bg: rgb(0x2d3139).into(), // #2d3139
            assistant_message_text: rgb(0xdcdfe4).into(), // #dcdfe4

            button_bg: rgb(0x41464f).into(),               // #41464f
            button_bg_hover: rgb(0x4b505a).into(),         // #4b505a
            button_text: rgb(0xdcdfe4).into(),             // #dcdfe4
            button_primary_bg: rgb(0x61afef).into(),       // #61afef
            button_primary_bg_hover: rgb(0x4b9bdc).into(), // #4b9bdc
            button_primary_text: white(),

            input_bg: rgb(0x373b45).into(),     // #373b45
            input_border: rgb(0x41454f).into(), // #41454f
            input_text: rgb(0xdcdfe4).into(),   // #dcdfe4
        }
    }

    pub fn from_theme_type(theme_type: &ThemeType) -> Self {
        match theme_type {
            ThemeType::OneLight => GpuiTheme::one_light(),
            ThemeType::OneDark => GpuiTheme::one_dark(),
        }
    }
}
