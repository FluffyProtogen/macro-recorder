pub mod delay_modify_command_window;
pub mod keyboard_command_window;
pub mod mouse_modify_command_window;
pub mod wait_for_image_modify_command_window;

use std::rc::Rc;

use crate::actions::Action;
use crate::ModalWindow;
use eframe::egui::*;

use self::delay_modify_command_window::DelayModifyCommandWindow;
use self::keyboard_command_window::KeyboardModifyCommandWindow;
use self::mouse_modify_command_window::MouseModifyCommandWindow;
use self::wait_for_image_modify_command_window::WaitForImageModifyCommandWindow;

impl Action {
    pub fn get_modify_command_window(
        &self,
        creating_command: bool,
        position: Pos2,
        ctx: &Context,
    ) -> Rc<dyn ModalWindow> {
        match self {
            Self::Mouse(mouse_action_kind) => Rc::new(MouseModifyCommandWindow::new(
                creating_command,
                position,
                mouse_action_kind,
            )),
            Self::Delay(delay) => Rc::new(DelayModifyCommandWindow::new(
                creating_command,
                position,
                *delay,
            )),
            Self::Keyboard(key, key_state) => Rc::new(KeyboardModifyCommandWindow::new(
                creating_command,
                position,
                *key,
                *key_state,
            )),
            Self::WaitForImage(image_info) => Rc::new(WaitForImageModifyCommandWindow::new(
                image_info,
                creating_command,
                position,
                ctx,
            )),
        }
    }
}
