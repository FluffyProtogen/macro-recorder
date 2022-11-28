//#![windows_subsystem = "windows"]
use eframe::*;
use egui::vec2;
use macro_recorder::*;

/*
    ALSO MAKE SPECIAL CASE FOR IT IF IT USES 1 SIMILARITY. MAKE IT CHECK EACH PIXEL INSTEAD OF MATCH_TEMPLATE
    ADD WHILE KEY HELD

    IF KEY PRESSED
    ADD FOREVER LOOP, ADD REPEAT, ADD END LOOP, ADD BREAK
    ADD CONSUME HOTKEY PRESSES (ONLY WORKS FOR SINGLE PRESS HOTKEYS OR CTRL / ALT)
    MAKE IT SO THAT IF YOU CLICK ON THE DROP DOWN FOR THE KEYS YOU CAN PRESS A KEY AND IT WILL AUTOMATICALLY SELECT IT
*/

fn main() {
    let mut options = NativeOptions::default();
    options.initial_window_size = Some(vec2(800.0, 650.0));
    options.min_window_size = Some(vec2(800.0, 650.0));
    options.transparent = true;
    run_native(
        "Fluffy Macro Recorder",
        options,
        Box::new(|cc| Box::new(gui::Recorder::new(cc))),
    );
}
