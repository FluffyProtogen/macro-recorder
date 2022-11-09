//#![windows_subsystem = "windows"]
use eframe::*;
use egui::vec2;
use macro_recorder::*;

/*
    NEED TO MAKE A MOVE / RECORD MOUSE POSITION CHANGE INSTEAD OF ABSOLUTE POSITION
    ALSO MAKE SPECIAL CASE FOR IT IF IT USES 1 SIMILARITY. MAKE IT CHECK EACH PIXEL INSTEAD OF MATCH_TEMPLATE
    ADD WHILE KEY HELD
    duration is 18 ms
*/

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
