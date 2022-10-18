use serde::*;

use crate::{images::RawScreenshotPair, keycodes_to_string::key_code_to_string};
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize)]
pub enum MouseActionKind {
    Moved(Point),
    Button(MouseActionButton),
    Wheel(i32, Option<Point>),
}

#[derive(Serialize, Deserialize)]
pub struct MouseActionButton {
    pub point: Option<Point>,
    pub button: i32,
    pub state: MouseActionButtonState,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum MouseActionButtonState {
    Pressed,
    Released,
    Clicked,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Delay(u32),
    Mouse(MouseActionKind),
    Keyboard(i32, KeyState),
    WaitForImage(ImageInfo),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum KeyState {
    Down,
    Up,
    Pressed,
}

#[derive(Serialize, Deserialize)]
pub struct ImageInfo {
    pub screenshot_raw: Option<RawScreenshotPair>,
    pub move_mouse_if_found: bool,
    pub check_if_not_found: bool,
    pub search_location_left_top: Option<(i32, i32)>,
    pub search_location_width_height: Option<(i32, i32)>,
    pub image_similarity: f32,
}

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            screenshot_raw: None,
            move_mouse_if_found: false,
            check_if_not_found: false,
            search_location_left_top: None,
            search_location_width_height: None,
            image_similarity: 0.0,
        }
    }
}

impl Action {
    pub fn get_grid_formatted(&self) -> [String; 3] {
        match self {
            Self::Delay(delay) => ["Delay".into(), delay.to_string(), "".into()],
            Self::Mouse(kind) => match kind {
                MouseActionKind::Moved(point) => [
                    "Mouse".into(),
                    "Moved".into(),
                    format!("X = {}, Y = {}", point.x, point.y),
                ],
                MouseActionKind::Button(action_button) => [
                    "Mouse".into(),
                    format!(
                        "Button {} {}",
                        key_code_to_string(action_button.button),
                        match action_button.state {
                            MouseActionButtonState::Pressed => "Down",
                            MouseActionButtonState::Released => "Up",
                            MouseActionButtonState::Clicked => "Clicked",
                        }
                    ),
                    match action_button.point {
                        Some(point) => format!("X = {}, Y = {}", point.x, point.y),
                        None => "Current Position".into(),
                    },
                ],
                MouseActionKind::Wheel(amount, _) => {
                    ["Mouse".into(), "Wheel".into(), (amount / 120).to_string()]
                }
            },
            Self::Keyboard(key_code, state) => [
                "Keyboard".into(),
                format!("Key {}", key_code_to_string(*key_code)),
                format!("{:?}", state),
            ],
            Self::WaitForImage(image_info) => [
                "Wait For Image".into(),
                if image_info.check_if_not_found {
                    "Wait until not found".into()
                } else {
                    "Wait until found".into()
                },
                if image_info.move_mouse_if_found {
                    "Move mouse to center if found".into()
                } else {
                    "".into()
                },
            ],
        }
    }
}
