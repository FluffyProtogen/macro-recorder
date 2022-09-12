use std::ffi::c_void;

//#![windows_subsystem = "windows"]
use eframe::*;
use egui::vec2;
use image::{ImageBuffer, ImageFormat, Rgba};
use macro_recorder::{images::image_capture_overlay, *};
use winapi::um::{wingdi::*, winuser::*};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

fn main() {
    let mut options = NativeOptions::default();
    options.initial_window_size = Some(vec2(1440.0, 1040.0));
    options.min_window_size = Some(vec2(800.0, 800.0));
    options.transparent = true;
    run_native(
        "Fluffy Macro Recorder",
        options,
        Box::new(|cc| Box::new(gui::Recorder::new(cc, vec![]))),
    );
}
