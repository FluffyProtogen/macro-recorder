pub mod actions;
pub mod gui;
pub mod images;
pub mod keycodes_to_string;
pub mod modals;
pub mod recorder;
pub mod right_click_dialog;
pub mod settings;
use actions::*;
use chrono::{DateTime, Utc};
use egui::pos2;
use images::find_image;
use settings::Settings;
use std::{
    error::Error,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    time::SystemTime,
};

use winapi::um::winuser::*;

fn execute_mouse_action(action: &MouseActionButton) {
    if action.state == MouseActionButtonState::Pressed
        || action.state == MouseActionButtonState::Released
    {
        let flag = match action.button {
            VK_LBUTTON => {
                if action.state == MouseActionButtonState::Pressed {
                    MOUSEEVENTF_LEFTDOWN
                } else {
                    MOUSEEVENTF_LEFTUP
                }
            }
            VK_RBUTTON => {
                if action.state == MouseActionButtonState::Pressed {
                    MOUSEEVENTF_RIGHTDOWN
                } else {
                    MOUSEEVENTF_RIGHTUP
                }
            }
            VK_MBUTTON => {
                if action.state == MouseActionButtonState::Pressed {
                    MOUSEEVENTF_MIDDLEDOWN
                } else {
                    MOUSEEVENTF_MIDDLEUP
                }
            }
            _ => panic!(
            "Somehow got a mouse button other than left / middle / right in execute mouse action"
        ),
        };

        let mouse_input = MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: flag,
            dwExtraInfo: 0,
            time: 0,
        };

        let mut input = INPUT {
            type_: INPUT_MOUSE,
            u: unsafe { std::mem::transmute_copy(&mouse_input) },
        };

        if let Some(point) = action.point {
            unsafe { SetCursorPos(point.x, point.y) };
        }

        unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };
    } else {
        let mut inputs = [
            INPUT {
                type_: INPUT_MOUSE,
                u: unsafe {
                    std::mem::transmute_copy(&MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: match action.button {
                    VK_LBUTTON => MOUSEEVENTF_LEFTDOWN,
                    VK_RBUTTON => MOUSEEVENTF_RIGHTDOWN,
                    VK_MBUTTON => MOUSEEVENTF_MIDDLEDOWN,
                    _ => panic!("Somehow got a mouse button other than left / middle / right in execute mouse action"),
            },
                dwExtraInfo: 0,
                time: 0,
            })
                },
            },
            INPUT {
                type_: INPUT_MOUSE,
                u: unsafe {
                    std::mem::transmute_copy(&MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: match action.button {
                    VK_LBUTTON => MOUSEEVENTF_LEFTUP,
                    VK_RBUTTON => MOUSEEVENTF_RIGHTUP,
                    VK_MBUTTON => MOUSEEVENTF_MIDDLEUP,
                    _ => panic!("Somehow got a mouse button other than left / middle / right in execute mouse action"),
            },
                dwExtraInfo: 0,
                time: 0,
            })
                },
            },
        ];

        if let Some(point) = action.point {
            unsafe { SetCursorPos(point.x, point.y) };
        }

        unsafe { SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32) };
    }
}

fn execute_keyboard_action(key_code: i32, state: KeyState) {
    if state != KeyState::Pressed {
        let mut keybd_input: INPUT_u = unsafe { std::mem::zeroed() };
        unsafe {
            *keybd_input.ki_mut() = KEYBDINPUT {
                wVk: key_code as u16,
                dwExtraInfo: 0,
                wScan: 0,
                time: 0,
                dwFlags: if state == KeyState::Down {
                    0
                } else {
                    KEYEVENTF_KEYUP
                },
            };
        };
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: keybd_input,
        };

        unsafe {
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        };
    } else {
        let mut inputs = [
            INPUT {
                type_: INPUT_KEYBOARD,
                u: unsafe {
                    let mut keybd_input: INPUT_u = std::mem::zeroed();
                    *keybd_input.ki_mut() = KEYBDINPUT {
                        wVk: key_code as u16,
                        dwExtraInfo: 0,
                        wScan: 0,
                        time: 0,
                        dwFlags: 0,
                    };
                    keybd_input
                },
            },
            INPUT {
                type_: INPUT_KEYBOARD,
                u: unsafe {
                    let mut keybd_input: INPUT_u = std::mem::zeroed();
                    *keybd_input.ki_mut() = KEYBDINPUT {
                        wVk: key_code as u16,
                        dwExtraInfo: 0,
                        wScan: 0,
                        time: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                    };
                    keybd_input
                },
            },
        ];

        unsafe { SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32) };
    }
}

pub fn play_back_actions(action_list: &[Action], settings: &Settings) {
    let mut if_stack: Vec<bool> = vec![];

    let mut counter = 0;
    while counter < settings.repeat_times || settings.repeat_times == 0 {
        for action in action_list.iter() {
            if stop_key_pressed() {
                return;
            }

            match action {
                Action::Else => {
                    if let Some(last) = if_stack.last_mut() {
                        *last = !*last;
                    } else {
                        panic!("NEED TO MAKE IT STOP THE PLAYBACK PROCESS (RETURN) AND MAKE AN ERROR WINDOW SAYING ELSE WITHOUT IF");
                    }
                }
                Action::EndIf => {
                    if if_stack.pop().is_none() {
                        panic!("NEED TO MAKE IT STOP THE PLAYBACK PROCESS (RETURN) AND MAKE AN ERROR WINDOW SAYING ENDIF WITHOUT IF");
                    }
                }
                _ => {}
            }

            if let Some(false) = if_stack.last() {
                continue;
            }

            match action {
                Action::Keyboard(key_code, state) => execute_keyboard_action(*key_code, *state),
                Action::Delay(delay) => {
                    if settings.ignore_delays {
                        continue;
                    }

                    let delay = *delay as f64 / settings.playback_speed as f64;

                    let time_started = DateTime::<Utc>::from(SystemTime::now());
                    while (DateTime::<Utc>::from(SystemTime::now()) - time_started)
                        .num_milliseconds()
                        < delay as i64
                    {
                        if stop_key_pressed() {
                            return;
                        }
                    }
                }

                Action::Mouse(action_kind) => match action_kind {
                    MouseActionKind::Moved(point) => unsafe {
                        SetCursorPos(point.x, point.y);
                    },
                    MouseActionKind::Button(action) => execute_mouse_action(action),
                    MouseActionKind::Wheel(amount, point) => execute_scroll_wheel(*amount, *point),
                },

                Action::WaitForImage(image_info) => execute_wait_for_image(image_info),
                Action::IfImage(image_info) => if_stack.push(execute_if_image(image_info)),
                Action::Else => {}
                Action::EndIf => {}
            }
        }

        if settings.repeat_times != 0 {
            counter += 1;
        }
    }
}

fn execute_if_image(image: &ImageInfo) -> bool {
    let search_coordinates = match (
        image.search_location_left_top,
        image.search_location_width_height,
    ) {
        (Some(left_top), Some(width_height)) => {
            let corner1 = pos2(left_top.0 as f32, left_top.1 as f32);
            let corner2 = pos2(
                corner1.x + width_height.0 as f32,
                corner1.y + width_height.1 as f32,
            );

            Some((corner1, corner2))
        }
        _ => None,
    };

    let (difference, (x, y)) =
        find_image(image.screenshot_raw.as_ref().unwrap(), search_coordinates);

    if image.check_if_not_found {
        if difference > image.image_similarity {
            true
        } else {
            false
        }
    } else {
        if difference <= image.image_similarity {
            println!("{difference}");
            if image.move_mouse_if_found {
                unsafe { SetCursorPos(x, y) };
            }
            true
        } else {
            false
        }
    }
}

fn execute_wait_for_image(image: &ImageInfo) {
    loop {
        if stop_key_pressed() || execute_if_image(image) {
            break;
        }
    }
}

fn execute_scroll_wheel(amount: i32, point: Option<Point>) {
    let mouse_input = MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: amount as u32,
        dwFlags: MOUSEEVENTF_WHEEL,
        dwExtraInfo: 0,
        time: 0,
    };

    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe { std::mem::transmute_copy(&mouse_input) },
    };

    if let Some(point) = point {
        unsafe { SetCursorPos(point.x, point.y) };
    }

    unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };
}

pub fn stop_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_CONTROL) < 0 && GetAsyncKeyState(0x51) < 0 }
}

pub fn play_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_CONTROL) < 0 && GetAsyncKeyState(0x50) < 0 }
}

pub fn load_from_file(path: &Path) -> Result<Vec<Action>, Box<dyn Error>> {
    let actions = read_to_string(path)?;
    let actions = serde_json::from_str(&actions)?;
    Ok(actions)
}

pub fn save_macro(path: &Path, action_list: &[Action]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;

    let action_list = serde_json::to_string(action_list).unwrap();

    file.write_all(action_list.as_bytes())?;

    Ok(())
}
