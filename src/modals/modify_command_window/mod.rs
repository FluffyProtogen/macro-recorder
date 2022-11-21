pub mod delay_modify_command_window;
pub mod image_modify_command_window;
pub mod keyboard_command_window;
pub mod mouse_modify_command_window;
pub mod pixel_modify_command_window;
pub mod repeat_modify_command_window;

use std::rc::Rc;

use crate::actions::Action;
use crate::modals::ModalWindow;
use eframe::egui::*;

use self::delay_modify_command_window::DelayModifyCommandWindow;
use self::image_modify_command_window::{ImageModifyCommandWindow, ImageWindowType};
use self::keyboard_command_window::KeyboardModifyCommandWindow;
use self::mouse_modify_command_window::MouseModifyCommandWindow;
use self::pixel_modify_command_window::{PixelModifyCommandWindow, PixelWindowType};
use self::repeat_modify_command_window::RepeatModifyCommandWindow;

impl Action {
    pub fn get_modify_command_window(
        &self,
        creating_command: bool,
        position: Pos2,
        ctx: &Context,
    ) -> Option<Rc<dyn ModalWindow>> {
        match self {
            Self::Mouse(mouse_action_kind) => Some(Rc::new(MouseModifyCommandWindow::new(
                creating_command,
                position,
                mouse_action_kind,
            ))),
            Self::Delay(delay) => Some(Rc::new(DelayModifyCommandWindow::new(
                creating_command,
                position,
                *delay,
            ))),
            Self::Keyboard(key, key_state) => Some(Rc::new(KeyboardModifyCommandWindow::new(
                creating_command,
                position,
                *key,
                *key_state,
            ))),
            Self::WaitForImage(image_info) => Some(Rc::new(ImageModifyCommandWindow::new(
                image_info,
                creating_command,
                position,
                ctx,
                ImageWindowType::Wait,
            ))),
            Self::IfImage(image_info) => Some(Rc::new(ImageModifyCommandWindow::new(
                image_info,
                creating_command,
                position,
                ctx,
                ImageWindowType::If,
            ))),

            Self::IfPixel(pixel_info) => Some(Rc::new(PixelModifyCommandWindow::new(
                pixel_info,
                creating_command,
                position,
                PixelWindowType::If,
            ))),

            Self::WaitForPixel(pixel_info) => Some(Rc::new(PixelModifyCommandWindow::new(
                pixel_info,
                creating_command,
                position,
                PixelWindowType::Wait,
            ))),

            Self::Repeat(times) => Some(Rc::new(RepeatModifyCommandWindow::new(
                creating_command,
                position,
                *times,
            ))),

            Self::Else | Self::EndIf | Self::EndRepeat | Self::Break => None,
        }
    }
}
