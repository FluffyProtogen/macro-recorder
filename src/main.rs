//OwOOwOOwOOwOaabcaOwO#![windows_subsystem = "windows"]

use eframe::{epaint::Vec2, *};
use macro_recorder::{keycodes_to_string::key_code_to_string, *};
fn main() {
    let action_list = serde_json::from_str::<Vec<_>>(r#""#).unwrap();

    //let action_list = record_actions(true);

    //while !play_key_pressed() {}

    //play_back_actions(&action_list);

    // while !play_key_pressed() {}ro

    // loop {
    //     play_back_actions(&action_list);

    //     if stop_key_pressed() {
    //         while !play_key_pressed() {}
    //     }
    // }

    //println!("{}", serde_json::to_string(&action_list).unwrap());

    let mut options = NativeOptions::default();
    options.initial_window_size = Some(eframe::egui::Vec2::new(1440.0, 1040.0));
    options.min_window_size = Some(Vec2 { x: 700.0, y: 700.0 });
    run_native(
        "Fluffy Protogens",
        options,
        Box::new(|cc| Box::new(gui::Recorder::new(cc, action_list))),
    );
}
