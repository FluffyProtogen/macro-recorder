//#![windows_subsystem = "windows"]
use eframe::*;
use egui::vec2; 
use macro_recorder::*;

fn main() {
    let mut options = NativeOptions::default();
    options.initial_window_size = Some(vec2(800.0, 650.0));
    options.min_window_size = Some(vec2(800.0, 650.0));
    options.transparent = true;
    run_native(
        "Fluffy Macro Recorder",
        options,
        Box::new(|cc| Box::new(gui::Recorder::new(cc, vec![]))),
    );
}
