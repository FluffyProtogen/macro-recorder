use serde::*;
use winapi::shared::windef::*;
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize)]
pub enum MouseActionKind {
    Moved(Point),
    Button(MouseActionButton),
}

#[derive(Serialize, Deserialize)]
pub struct MouseActionButton {
    pub point: Option<Point>,
    pub button: i32,
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Delay(u64),
    Mouse(MouseActionKind),
    Keyboard(i32, bool),
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
                        action_button.button,
                        if action_button.pressed {
                            "Pressed"
                        } else {
                            "Released"
                        }
                    ),
                    match action_button.point {
                        Some(point) => format!("X = {}, Y = {}", point.x, point.y),
                        None => "Current Position".into(),
                    },
                ],
            },
            Self::Keyboard(key_code, pressed) => [
                "Keyboard".into(),
                format!("Key {}", key_code),
                if *pressed {
                    "Pressed".into()
                } else {
                    "Released".into()
                },
            ],
        }
    }
}
