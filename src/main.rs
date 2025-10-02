mod api;
mod app;
mod attachments;
mod export;
mod storage;
mod theme;
mod tokens;
mod types;
mod ui;

use app::OverlookApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
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
        Box::new(|cc| Ok(Box::new(OverlookApp::new(cc)))),
    )
}
