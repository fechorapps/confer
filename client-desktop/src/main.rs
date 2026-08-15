mod app;
pub mod media;
mod sdk;
mod ui;

use app::ConferApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Confer — High Performance Video Conferencing")
            .with_maximized(true)
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Confer",
        native_options,
        Box::new(|cc| Ok(Box::new(ConferApp::new(cc)))),
    )
}
