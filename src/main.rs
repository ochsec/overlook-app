mod api;
mod attachments;
mod export;
mod markdown;
mod storage;
mod gpui_theme;
mod tokens;
mod tools;
mod types;

// New GPUI modules
mod components;
mod gpui_app;
mod license;
mod mcp;
mod state;
mod views;

use gpui::*;
use gpui_app::OverlookApp;
use gpui_component::theme::{Theme, ThemeMode};
use gpui_component::*;
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
