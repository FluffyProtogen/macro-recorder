pub mod actions;
pub mod gui;
pub mod keycodes_to_string;
pub mod modify_command_window;
pub mod recorder;
pub mod right_click_dialog;
pub mod settings;
pub mod settings_window;
pub mod warning_window;
use actions::*;
use chrono::{DateTime, Utc};
use settings::Settings;
use std::time::SystemTime;
use winapi::um::winuser::*;

fn execute_mouse_action(action: &MouseActionButton) {
    let flag = match action.button {
        VK_LBUTTON => {
            if action.pressed {
                MOUSEEVENTF_LEFTDOWN
            } else {
                MOUSEEVENTF_LEFTUP
            }
        }
        VK_RBUTTON => {
            if action.pressed {
                MOUSEEVENTF_RIGHTDOWN
            } else {
                MOUSEEVENTF_RIGHTUP
            }
        }
        VK_MBUTTON => {
            if action.pressed {
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
        unsafe {
            SetCursorPos(point.x, point.y);
        };
    }

    unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };
}

fn execute_keyboard_action(key_code: i32, state: bool) {
    let keybd_input = KEYBDINPUT {
        wVk: key_code as u16,
        dwExtraInfo: 0,
        wScan: 0,
        time: 0,
        dwFlags: if state { 0 } else { KEYEVENTF_KEYUP },
    };

    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe { std::mem::transmute_copy(&keybd_input) },
    };

    unsafe {
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    };
}

pub fn play_back_actions(action_list: &[Action], settings: &Settings) {
    let mut counter = 0;
    while counter < settings.repeat_times || settings.repeat_times == 0 {
        for action in action_list.iter() {
            if stop_key_pressed() {
                return;
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
            }
        }

        if settings.repeat_times != 0 {
            counter += 1;
        }
    }
}

pub fn execute_scroll_wheel(amount: i32, point: Option<Point>) {
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
        unsafe {
            SetCursorPos(point.x, point.y);
        };
    }

    unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };
}

pub fn stop_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_CONTROL) < 0 && GetAsyncKeyState(0x51) < 0 }
}

pub fn play_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_CONTROL) < 0 && GetAsyncKeyState(0x50) < 0 }
}
