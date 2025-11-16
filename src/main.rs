// GPUI Migration - Phase 0: Basic Hello World
// Original egui code preserved in git history (main branch)

use gpui::*;
use gpui_component::*;

// Temporary minimal app struct for testing GPUI setup
pub struct OverlookApp;

impl Render for OverlookApp {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .text_size(px(24.0))
            .child("GPUI Migration - Hello World ✓")
    }
}

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let app = Application::new();

    app.run(move |cx| {
        // Initialize gpui-component first
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(100.0), px(100.0)),
                        size: size(px(1000.0), px(700.0)),
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Overlook - GPUI Migration".into()),
                        appears_transparent: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|_| OverlookApp);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
