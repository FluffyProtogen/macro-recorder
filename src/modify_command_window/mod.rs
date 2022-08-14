pub mod delay_modify_command_window;
pub mod keyboard_command_window;
pub mod mouse_modify_command_window;

use std::cell::RefCell;

use crate::{
    actions::{self, Action},
    gui::Recorder,
};
use eframe::{egui::*, *};

use self::delay_modify_command_window::DelayModifyCommandWindow;
use self::keyboard_command_window::KeyboardModifyCommandWindow;
use self::mouse_modify_command_window::MouseModifyCommandWindow;

pub trait ModifyCommandWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        screen_dimensions: Rect,
        frame: &mut eframe::Frame,
    ) {
    }
}

impl Action {
    pub fn get_modify_command_window(
        &self,
        creating_command: bool,
        position: Pos2,
    ) -> Box<dyn ModifyCommandWindow> {
        match self {
            Self::Mouse(mouse_action_kind) => Box::new(MouseModifyCommandWindow::new(
                creating_command,
                position,
                mouse_action_kind,
            )),
            Self::Delay(delay) => Box::new(DelayModifyCommandWindow::new(
                creating_command,
                position,
                *delay,
            )),
            Self::Keyboard(key, key_state) => Box::new(KeyboardModifyCommandWindow::new(
                creating_command,
                position,
                *key,
                *key_state,
            )),
        }
    }
}
