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
            background: egui::Color32::from_rgb(250, 250, 250), // #fafafa
            surface: egui::Color32::from_rgb(255, 255, 255),    // #ffffff
            sidebar_background: egui::Color32::from_rgb(245, 245, 245), // #f5f5f5

            text_primary: egui::Color32::from_rgb(56, 58, 66), // #383a42
            text_secondary: egui::Color32::from_rgb(106, 115, 125), // #6a737d
            text_muted: egui::Color32::from_rgb(160, 161, 167), // #a0a1a7

            accent: egui::Color32::from_rgb(64, 120, 242), // #4078f2
            accent_hover: egui::Color32::from_rgb(45, 100, 220), // #2d64dc
            border: egui::Color32::from_rgb(228, 229, 233), // #e4e5e9
            separator: egui::Color32::from_rgb(234, 236, 239), // #eaecef

            user_message_bg: egui::Color32::from_rgb(64, 120, 242), // #4078f2
            user_message_text: egui::Color32::WHITE,
            assistant_message_bg: egui::Color32::from_rgb(240, 240, 240), // #f0f0f0
            assistant_message_text: egui::Color32::from_rgb(56, 58, 66),  // #383a42

            button_bg: egui::Color32::from_rgb(255, 255, 255), // #ffffff
            button_bg_hover: egui::Color32::from_rgb(245, 245, 245), // #f5f5f5
            button_text: egui::Color32::from_rgb(56, 58, 66),  // #383a42
            button_primary_bg: egui::Color32::from_rgb(64, 120, 242), // #4078f2
            button_primary_bg_hover: egui::Color32::from_rgb(45, 100, 220), // #2d64dc
            button_primary_text: egui::Color32::WHITE,

            input_bg: egui::Color32::from_rgb(255, 255, 255), // #ffffff
            input_border: egui::Color32::from_rgb(228, 229, 233), // #e4e5e9
            input_text: egui::Color32::from_rgb(56, 58, 66),  // #383a42
        }
    }

    pub fn one_dark() -> Self {
        Theme {
            // One Dark - Based on Atom's One Dark theme with improved contrast
            background: egui::Color32::from_rgb(40, 44, 52), // #282c34
            surface: egui::Color32::from_rgb(33, 37, 43),    // #21252b
            sidebar_background: egui::Color32::from_rgb(35, 39, 46), // #23272e

            text_primary: egui::Color32::from_rgb(220, 223, 228), // #dcdfe4 - Much brighter for better contrast
            text_secondary: egui::Color32::from_rgb(180, 187, 197), // #b4bbc5 - Brighter secondary text
            text_muted: egui::Color32::from_rgb(130, 137, 151), // #828997 - Slightly brighter muted text

            accent: egui::Color32::from_rgb(97, 175, 239), // #61afef
            accent_hover: egui::Color32::from_rgb(75, 155, 220), // #4b9bdc
            border: egui::Color32::from_rgb(55, 59, 69),   // #373b45
            separator: egui::Color32::from_rgb(50, 54, 64), // #323640

            user_message_bg: egui::Color32::from_rgb(97, 175, 239), // #61afef
            user_message_text: egui::Color32::WHITE, // Pure white for maximum contrast
            assistant_message_bg: egui::Color32::from_rgb(45, 49, 57), // #2d3139 - Slightly lighter background
            assistant_message_text: egui::Color32::from_rgb(220, 223, 228), // #dcdfe4 - Much brighter text

            button_bg: egui::Color32::from_rgb(65, 70, 80), // #41464f - Lighter for better contrast
            button_bg_hover: egui::Color32::from_rgb(75, 80, 90), // #4b505a - Even lighter on hover
            button_text: egui::Color32::from_rgb(220, 223, 228), // #dcdfe4 - Bright button text
            button_primary_bg: egui::Color32::from_rgb(97, 175, 239), // #61afef
            button_primary_bg_hover: egui::Color32::from_rgb(75, 155, 220), // #4b9bdc
            button_primary_text: egui::Color32::WHITE,      // Pure white for primary buttons

            input_bg: egui::Color32::from_rgb(55, 59, 69), // #373b45
            input_border: egui::Color32::from_rgb(65, 69, 79), // #41454f
            input_text: egui::Color32::from_rgb(220, 223, 228), // #dcdfe4 - Brighter input text
        }
    }

    pub fn from_theme_type(theme_type: &ThemeType) -> Self {
        match theme_type {
            ThemeType::OneLight => Theme::one_light(),
            ThemeType::OneDark => Theme::one_dark(),
        }
    }

    pub fn apply_to_egui_style(&self, style: &mut egui::Style, font_size: f32) {
        // Start with appropriate base theme (dark or light)
        let base_visuals = if matches!(self, Self { background, .. } if background.r() < 128) {
            egui::Visuals::dark() // Use dark base for dark themes
        } else {
            egui::Visuals::light() // Use light base for light themes
        };

        // CRITICAL: For light theme, ensure text_color() returns dark text
        // DO NOT set override_text_color here - it interferes with per-message colors

        style.visuals = base_visuals;

        // Apply theme colors to egui's visual style
        let visuals = &mut style.visuals;

        // Basic colors - Set ALL text color fields for proper contrast
        visuals.weak_text_color = Some(self.text_secondary);  // Secondary/muted text
        visuals.panel_fill = self.surface;
        visuals.window_fill = self.surface;
        visuals.faint_bg_color = self.sidebar_background;

        // Set text cursor color
        visuals.text_cursor.stroke.color = self.text_primary;

        // Widget colors - Fix contrast issues
        // Noninteractive widgets (labels, etc.)
        visuals.widgets.noninteractive.bg_fill = self.surface;
        visuals.widgets.noninteractive.fg_stroke.color = self.text_primary;
        visuals.widgets.noninteractive.bg_stroke.color = self.border;

        // Inactive widgets (buttons, dropdowns in default state)
        visuals.widgets.inactive.bg_fill = self.button_bg;
        visuals.widgets.inactive.fg_stroke.color = self.text_primary;
        visuals.widgets.inactive.bg_stroke.color = self.border;

        // Hovered widgets (buttons, dropdowns when mouse over)
        visuals.widgets.hovered.bg_fill = self.button_bg_hover;
        visuals.widgets.hovered.fg_stroke.color = self.text_primary;
        visuals.widgets.hovered.bg_stroke.color = self.accent;

        // Active/pressed widgets
        // CRITICAL: active.fg_stroke.color is used by strong_text_color() for markdown bold/headings!
        // It should be high-contrast text on accent background, BUT also work for inline strong text
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.fg_stroke.color = self.text_primary; // Use theme text color, not white!
        visuals.widgets.active.bg_stroke.color = self.accent;

        // Open widgets (like opened ComboBoxes)
        visuals.widgets.open.bg_fill = self.button_bg_hover;
        visuals.widgets.open.fg_stroke.color = self.text_primary;
        visuals.widgets.open.bg_stroke.color = self.accent;

        // Selection color
        visuals.selection.bg_fill = self.accent;
        visuals.selection.stroke.color = self.accent;

        // Separator color
        visuals.widgets.noninteractive.bg_stroke.color = self.separator;

        // Extreme colors
        visuals.extreme_bg_color = self.background;

        // Hyperlink color
        visuals.hyperlink_color = self.accent;

        // Code background for better text contrast
        visuals.code_bg_color = self.assistant_message_bg;

        // Text edit (input box) colors - CRITICAL for light mode readability
        visuals.extreme_bg_color = self.input_bg;
        visuals.widgets.inactive.weak_bg_fill = self.input_bg;
        visuals.widgets.hovered.weak_bg_fill = self.input_bg;
        visuals.widgets.active.weak_bg_fill = self.input_bg;

        // Window shadow
        visuals.window_shadow.color = egui::Color32::from_black_alpha(100);

        // Apply font size
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(font_size - 2.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(font_size, egui::FontFamily::Monospace),
        );
    }
}
