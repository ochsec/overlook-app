use eframe::egui;

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
pub struct Theme {
    // Background colors
    pub background: egui::Color32,
    pub surface: egui::Color32,
    pub sidebar_background: egui::Color32,

    // Text colors
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
    pub text_muted: egui::Color32,

    // UI element colors
    pub accent: egui::Color32,
    pub accent_hover: egui::Color32,
    pub border: egui::Color32,
    pub separator: egui::Color32,

    // Message bubble colors
    pub user_message_bg: egui::Color32,
    pub user_message_text: egui::Color32,
    pub assistant_message_bg: egui::Color32,
    pub assistant_message_text: egui::Color32,

    // Button colors
    pub button_bg: egui::Color32,
    pub button_bg_hover: egui::Color32,
    pub button_text: egui::Color32,
    pub button_primary_bg: egui::Color32,
    pub button_primary_bg_hover: egui::Color32,
    pub button_primary_text: egui::Color32,

    // Input colors
    pub input_bg: egui::Color32,
    pub input_border: egui::Color32,
    pub input_text: egui::Color32,
}

impl Theme {
    pub fn one_light() -> Self {
        Theme {
            // One Light - Based on Atom's One Light theme
            background: egui::Color32::from_rgb(250, 250, 250),           // #fafafa
            surface: egui::Color32::from_rgb(255, 255, 255),              // #ffffff
            sidebar_background: egui::Color32::from_rgb(245, 245, 245),   // #f5f5f5

            text_primary: egui::Color32::from_rgb(56, 58, 66),            // #383a42
            text_secondary: egui::Color32::from_rgb(106, 115, 125),       // #6a737d
            text_muted: egui::Color32::from_rgb(160, 161, 167),           // #a0a1a7

            accent: egui::Color32::from_rgb(64, 120, 242),                // #4078f2
            accent_hover: egui::Color32::from_rgb(45, 100, 220),          // #2d64dc
            border: egui::Color32::from_rgb(228, 229, 233),               // #e4e5e9
            separator: egui::Color32::from_rgb(234, 236, 239),            // #eaecef

            user_message_bg: egui::Color32::from_rgb(64, 120, 242),       // #4078f2
            user_message_text: egui::Color32::WHITE,
            assistant_message_bg: egui::Color32::from_rgb(240, 240, 240), // #f0f0f0
            assistant_message_text: egui::Color32::from_rgb(56, 58, 66),  // #383a42

            button_bg: egui::Color32::from_rgb(255, 255, 255),            // #ffffff
            button_bg_hover: egui::Color32::from_rgb(245, 245, 245),      // #f5f5f5
            button_text: egui::Color32::from_rgb(56, 58, 66),             // #383a42
            button_primary_bg: egui::Color32::from_rgb(64, 120, 242),     // #4078f2
            button_primary_bg_hover: egui::Color32::from_rgb(45, 100, 220), // #2d64dc
            button_primary_text: egui::Color32::WHITE,

            input_bg: egui::Color32::from_rgb(255, 255, 255),             // #ffffff
            input_border: egui::Color32::from_rgb(228, 229, 233),         // #e4e5e9
            input_text: egui::Color32::from_rgb(56, 58, 66),              // #383a42
        }
    }

    pub fn one_dark() -> Self {
        Theme {
            // One Dark - Based on Atom's One Dark theme
            background: egui::Color32::from_rgb(40, 44, 52),              // #282c34
            surface: egui::Color32::from_rgb(33, 37, 43),                 // #21252b
            sidebar_background: egui::Color32::from_rgb(35, 39, 46),      // #23272e

            text_primary: egui::Color32::from_rgb(171, 178, 191),         // #abb2bf
            text_secondary: egui::Color32::from_rgb(130, 137, 151),       // #828997
            text_muted: egui::Color32::from_rgb(92, 99, 112),             // #5c6370

            accent: egui::Color32::from_rgb(97, 175, 239),                // #61afef
            accent_hover: egui::Color32::from_rgb(75, 155, 220),          // #4b9bdc
            border: egui::Color32::from_rgb(55, 59, 69),                  // #373b45
            separator: egui::Color32::from_rgb(50, 54, 64),               // #323640

            user_message_bg: egui::Color32::from_rgb(97, 175, 239),       // #61afef
            user_message_text: egui::Color32::from_rgb(40, 44, 52),       // #282c34
            assistant_message_bg: egui::Color32::from_rgb(55, 59, 69),    // #373b45
            assistant_message_text: egui::Color32::from_rgb(171, 178, 191), // #abb2bf

            button_bg: egui::Color32::from_rgb(55, 59, 69),               // #373b45
            button_bg_hover: egui::Color32::from_rgb(65, 69, 79),         // #41454f
            button_text: egui::Color32::from_rgb(171, 178, 191),          // #abb2bf
            button_primary_bg: egui::Color32::from_rgb(97, 175, 239),     // #61afef
            button_primary_bg_hover: egui::Color32::from_rgb(75, 155, 220), // #4b9bdc
            button_primary_text: egui::Color32::from_rgb(40, 44, 52),     // #282c34

            input_bg: egui::Color32::from_rgb(55, 59, 69),                // #373b45
            input_border: egui::Color32::from_rgb(65, 69, 79),            // #41454f
            input_text: egui::Color32::from_rgb(171, 178, 191),           // #abb2bf
        }
    }

    pub fn from_theme_type(theme_type: &ThemeType) -> Self {
        match theme_type {
            ThemeType::OneLight => Theme::one_light(),
            ThemeType::OneDark => Theme::one_dark(),
        }
    }

    pub fn apply_to_egui_style(&self, style: &mut egui::Style) {
        // Apply theme colors to egui's visual style
        let visuals = &mut style.visuals;

        // Basic colors
        visuals.override_text_color = Some(self.text_primary);
        visuals.panel_fill = self.surface;
        visuals.window_fill = self.surface;
        visuals.faint_bg_color = self.sidebar_background;

        // Widget colors
        visuals.widgets.noninteractive.bg_fill = self.button_bg;
        visuals.widgets.noninteractive.fg_stroke.color = self.text_primary;

        visuals.widgets.inactive.bg_fill = self.button_bg;
        visuals.widgets.inactive.fg_stroke.color = self.text_primary;
        visuals.widgets.inactive.bg_stroke.color = self.border;

        visuals.widgets.hovered.bg_fill = self.button_bg_hover;
        visuals.widgets.hovered.fg_stroke.color = self.text_primary;
        visuals.widgets.hovered.bg_stroke.color = self.accent;

        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.fg_stroke.color = self.button_primary_text;

        // Selection color
        visuals.selection.bg_fill = self.accent;
        visuals.selection.stroke.color = self.accent;

        // Separator color
        visuals.widgets.noninteractive.bg_stroke.color = self.separator;

        // Extreme colors
        visuals.extreme_bg_color = self.background;

        // Hyperlink color
        visuals.hyperlink_color = self.accent;
    }
}