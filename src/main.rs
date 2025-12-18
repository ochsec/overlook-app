// GPUI Migration - Phase 1: Infrastructure & View Routing
// Original egui code preserved in git history (main branch)

mod api;
mod attachments;
mod export;
mod markdown;  // New markdown renderer for GPUI
mod storage;
// mod theme;  // Old egui theme - backed up as reference
mod gpui_theme;  // New GPUI theme
mod tokens;
mod tools;
mod types;
// mod ui;  // Old egui UI - backed up as ui.rs.egui_backup

// New GPUI modules
mod components;
mod views;
mod state;
mod gpui_app;

use gpui::*;
use gpui_component::*;
use gpui_component::theme::{Theme, ThemeMode};
use gpui_app::OverlookApp;
use gpui_theme::ThemeType;

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let app = Application::new();

    app.run(move |cx| {
        // Initialize gpui-component first
        gpui_component::init(cx);

        // Sync gpui_component theme with our app theme
        let saved_theme = storage::Storage::load_theme().unwrap_or_default();
        let theme_mode = match saved_theme {
            ThemeType::OneLight => ThemeMode::Light,
            ThemeType::OneDark => ThemeMode::Dark,
        };
        Theme::change(theme_mode, None, cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(100.0), px(100.0)),
                        size: size(px(1200.0), px(800.0)),
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Overlook - OpenRouter Chat".into()),
                        appears_transparent: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| OverlookApp::new(cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
