use egui::{Context, Rect, Ui};

use crate::gui::Recorder;

pub mod action_list_category;
pub mod hotkeys_window;
pub mod modify_command_window;
pub mod settings_window;
pub mod warning_window;

pub trait ModalWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        screen_dimensions: Rect,
        frame: &mut eframe::Frame,
    );
}
