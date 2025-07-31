pub use egui;

pub mod app;
pub use app::*;

pub mod node;
pub use node::*;


fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 1000.0])
            .with_min_inner_size([1000.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Gates",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}