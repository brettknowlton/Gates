use egui;

mod app;
use app::*;

mod node;
use node::*;



fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 8000.0])
            .with_min_inner_size([1000.0, 8000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Gates",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}






// use std::hash::Hash;
// use eframe::egui;
// use egui::CentralPanel;
// use egui_dnd::dnd;

// pub fn main() -> eframe::Result<()> {
//     let mut items = vec!["alfred", "bernhard", "christian"];

//     eframe::run_simple_native("DnD Simple Example", Default::default(), move |ctx, _frame| {
//         CentralPanel::default().show(ctx, |ui| {

//             ui.horizontal(|ui| {
//                 ui.label("Drag and drop example");
//                 dnd(ui, "dnd_example")
//                 .show_vec(&mut items, |ui, item, handle, state| {
//                     ui.vertical(|ui| {
//                         handle.ui(ui, |ui| {
//                             ui.label("drag");
//                         });
//                         ui.label(*item);
//                     });
//                 });
//             });
            

//         });
//     })
// }