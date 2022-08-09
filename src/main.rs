//#![windows_subsystem = "windows"]

use eframe::{epaint::Vec2, *};
use macro_recorder::*;
use serde::Deserialize;

fn main() {
    let mut options = NativeOptions::default();
    options.initial_window_size = Some(eframe::egui::Vec2::new(1440.0, 1040.0));
    options.min_window_size = Some(Vec2 { x: 800.0, y: 800.0 });
    run_native(
        "Fluffy Macro Recorder",
        options,
        Box::new(|cc| Box::new(gui::Recorder::new(cc, vec![]))),
    );
}
