//#![windows_subsystem = "windows"]

use eframe::{epaint::Vec2, *};
use macro_recorder::*;
fn main() {
    let mut options = NativeOptions::default();
    options.initial_window_size = Some(eframe::egui::Vec2::new(1440.0, 1040.0));
    options.min_window_size = Some(Vec2 { x: 700.0, y: 700.0 });
    run_native(
        "Fluffy Protogens",
        options,
        Box::new(|cc| Box::new(gui::Recorder::new(cc, vec![]))),
    );
}
