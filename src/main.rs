mod api;
mod app;
mod attachments;
mod export;
mod markdown_renderer;
mod storage;
mod theme;
mod tokens;
mod tools;
mod types;
mod ui;

use app::OverlookApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Overlook - OpenRouter Chat",
        options,
        Box::new(|cc| {
            // Load and apply theme BEFORE creating the app
            let theme_type = storage::Storage::load_theme().unwrap_or_default();
            let theme = theme::Theme::from_theme_type(&theme_type);
            let font_size = storage::Storage::load_font_size().unwrap_or(14.0);

            cc.egui_ctx.style_mut(|style| {
                theme.apply_to_egui_style(style, font_size);
            });

            // Install image loaders for egui-twemoji SVG rendering
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(OverlookApp::new(cc)))
        }),
    )
}
