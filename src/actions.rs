use serde::*;

use crate::keycodes_to_string::key_code_to_string;
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
    WaitForImage,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum KeyState {
    Down,
    Up,
    Pressed,
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
            Self::WaitForImage => ["IMAGE".into(), "IMAGE".into(), "IMAGE".into()],
        }
    }
}
