use std::{borrow::Cow, path::PathBuf};

use egui::Color32;
use serde::*;

use crate::{images::RawScreenshotPair, keycodes_to_string::key_code_to_string};
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MouseActionKind {
    Moved(MousePointKind),
    Button(MouseActionButton),
    Wheel(i32, Option<MousePointKind>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MouseActionButton {
    pub point: Option<MousePointKind>,
    pub button: i32,
    pub state: MouseActionButtonState,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum MousePointKind {
    To(Point),
    By(Point),
}

impl MousePointKind {
    pub fn x(self) -> i32 {
        match self {
            MousePointKind::To(point) => point.x,
            MousePointKind::By(point) => point.x,
        }
    }

    pub fn y(self) -> i32 {
        match self {
            MousePointKind::To(point) => point.y,
            MousePointKind::By(point) => point.y,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum MouseActionButtonState {
    Pressed,
    Released,
    Clicked,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Action {
    Delay(u32),
    Mouse(MouseActionKind),
    Keyboard(i32, KeyState),
    WaitForImage(ImageInfo),
    IfImage(ImageInfo),
    WaitForPixel(PixelInfo),
    IfPixel(PixelInfo),
    Else,
    EndIf,
    Repeat(usize),
    EndRepeat,
    Break,
    Play(PathBuf),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum KeyState {
    Down,
    Up,
    Pressed,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PixelInfo {
    pub color: (u8, u8, u8),
    pub search_location_left_top: (i32, i32),
    pub search_location_width_height: (i32, i32),
    pub check_if_not_found: bool,
    pub move_mouse_if_found: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
            image_similarity: 1.0,
        }
    }
}

impl Action {
    pub fn get_grid_formatted(&self) -> [String; 3] {
        match self {
            Self::Delay(delay) => ["Delay".into(), delay.to_string(), "".into()],
            Self::Mouse(kind) => match kind {
                MouseActionKind::Moved(point) => {
                    let (move_type, x, y) = match *point {
                        MousePointKind::To(point) => ("Moved To", point.x, point.y),
                        MousePointKind::By(point) => ("Changed By", point.x, point.y),
                    };

                    [
                        "Mouse".into(),
                        move_type.to_string(),
                        format!("X = {}, Y = {}", x, y),
                    ]
                }
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
                    )
                    .into(),
                    match action_button.point {
                        Some(point) => {
                            let (move_type, x, y) = match point {
                                MousePointKind::To(point) => ("At", point.x, point.y),
                                MousePointKind::By(point) => ("Moved By", point.x, point.y),
                            };
                            format!("{} X = {}, Y = {}", move_type, x, y)
                        }
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
            Self::IfImage(image_info) => [
                "If Image".into(),
                if image_info.check_if_not_found {
                    "If image not found".into()
                } else {
                    "If image found".into()
                },
                if image_info.move_mouse_if_found {
                    "Move mouse to center if found".into()
                } else {
                    "".into()
                },
            ],
            Self::Else => ["Else".into(), "".into(), "".into()],
            Self::EndIf => ["End If".into(), "".into(), "".into()],
            Self::IfPixel(info) => [
                "If pixel".into(),
                if info.check_if_not_found {
                    "If pixel not found".into()
                } else {
                    "If pixel found".into()
                },
                if info.move_mouse_if_found {
                    "Move mouse to center if found".into()
                } else {
                    "".into()
                },
            ],
            Self::WaitForPixel(info) => [
                "Wait For Pixel".into(),
                if info.check_if_not_found {
                    "Wait for no pixel".into()
                } else {
                    "Wait for pixel".into()
                },
                if info.move_mouse_if_found {
                    "Move mouse to center if found".into()
                } else {
                    "".into()
                },
            ],
            Self::Repeat(amount) => [
                "Repeat".into(),
                if *amount == 0 {
                    "Forever".into()
                } else {
                    format!("{amount} Times")
                },
                "".into(),
            ],
            Self::EndRepeat => ["End Repeat".into(), "".into(), "".into()],
            Self::Break => ["Break".into(), "".into(), "".into()],
            Self::Play(path) => ["Play".into(), path.to_string_lossy().into(), "".into()],
        }
    }
}
