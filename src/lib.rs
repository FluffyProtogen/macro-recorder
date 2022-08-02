pub mod actions;
pub mod gui;
use actions::*;
use chrono::{DateTime, Utc};
use serde::*;
use serde_json::*;
use std::time::Duration;
use std::time::SystemTime;
use winapi::shared::ntstatus::STATUS_QUERY_STORAGE_ERROR;
use winapi::{
    shared::{minwindef::*, windef::*},
    um::winuser::*,
};

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

fn get_key_states() -> Vec<bool> {
    (0..0xFE)
        .map(|key_code| unsafe { GetKeyState(key_code) & 0x80 != 0 })
        .collect()
}

fn get_changed_key_states(
    previous_key_states: &[bool],
    current_key_states: &[bool],
) -> Vec<(i32, bool)> {
    previous_key_states
        .iter()
        .zip(current_key_states)
        .enumerate()
        .filter_map(|(key_code, (previous_state, current_state))| {
            if *previous_state != *current_state {
                Some((key_code as i32, *current_state))
            } else {
                None
            }
        })
        .collect()
}

fn get_mouse_position() -> POINT {
    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut point) };
    point
}

fn update_action_list(
    action_list: &mut Vec<Action>,
    delay: u64,
    changed_key_states: &[(i32, bool)],
    current_mouse_position: POINT,
    previous_mouse_position: POINT,
    record_mouse_movement: bool,
) {
    if delay != 0 {
        action_list.push(Action::Delay(delay));
    }

    for (key_code, state) in changed_key_states.iter() {
        match *key_code {
            VK_LBUTTON | VK_RBUTTON | VK_MBUTTON => {
                action_list.push(Action::Mouse(MouseActionKind::Button(MouseActionButton {
                    point: Some(Point {
                        x: current_mouse_position.x,
                        y: current_mouse_position.y,
                    }),
                    button: *key_code as i32,
                    pressed: *state,
                })))
            }
            VK_CONTROL | VK_SHIFT | VK_MENU => (),
            _ => action_list.push(Action::Keyboard(*key_code as i32, *state)),
        }
    }

    if record_mouse_movement
        && (previous_mouse_position.x != current_mouse_position.x
            || previous_mouse_position.y != current_mouse_position.y)
    {
        action_list.push(Action::Mouse(MouseActionKind::Moved(Point {
            x: current_mouse_position.x,
            y: current_mouse_position.y,
        })));
    }
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

pub fn play_back_actions(action_list: &[Action]) {
    for action in action_list.iter() {
        if stop_key_pressed() {
            return;
        }

        match action {
            Action::Keyboard(key_code, state) => execute_keyboard_action(*key_code, *state),
            Action::Delay(delay) => {
                let time_started = DateTime::<Utc>::from(SystemTime::now());
                while (DateTime::<Utc>::from(SystemTime::now()) - time_started)
                    .num_microseconds()
                    .unwrap_or(0)
                    < *delay as i64
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
            },
        }
    }
}

pub fn record_actions(record_mouse_movement: bool) -> Vec<Action> {
    let mut previous_key_states = get_key_states();
    let mut previous_mouse_position = get_mouse_position();

    let mut action_list = vec![];

    let mut time_since_last_action = DateTime::<Utc>::from(SystemTime::now());

    while !stop_key_pressed() {
        let current_key_states = get_key_states();
        let current_mouse_position = get_mouse_position();

        let mut changed_key_states =
            get_changed_key_states(&previous_key_states, &current_key_states);

        changed_key_states.retain(|(key_code, _)| *key_code != VK_MENU);
        changed_key_states.retain(|(key_code, _)| *key_code != VK_SHIFT);
        changed_key_states.retain(|(key_code, _)| *key_code != VK_CONTROL);

        let delay = if changed_key_states.len() != 0
            || (record_mouse_movement
                && (previous_mouse_position.x != current_mouse_position.x
                    || previous_mouse_position.y != current_mouse_position.y))
        {
            let current_time = DateTime::<Utc>::from(SystemTime::now());

            let delay = (current_time - time_since_last_action)
                .num_microseconds()
                .unwrap_or(0) as u64;

            time_since_last_action = current_time;
            println!("F");
            delay
        } else {
            0
        };

        update_action_list(
            &mut action_list,
            delay,
            &changed_key_states,
            current_mouse_position,
            previous_mouse_position,
            record_mouse_movement,
        );

        previous_key_states = current_key_states;
        previous_mouse_position = current_mouse_position;
    }

    remove_action_list_stop_combination(&mut action_list);

    action_list //
}

fn remove_action_list_stop_combination(action_list: &mut Vec<Action>) {
    let position = action_list
        .iter()
        .rev()
        .position(|action| match *action {
            Action::Keyboard(key_code, state) => {
                state && (key_code == VK_LCONTROL || key_code == VK_RCONTROL)
            }
            _ => false,
        })
        .unwrap();

    let position = action_list.len() - position - 2;

    action_list.drain(position..action_list.len());
}

pub fn stop_key_pressed() -> bool {
    unsafe { GetKeyState(VK_CONTROL) & 0x80 != 0 && GetKeyState(0x51) & 0x80 != 0 }
}

pub fn play_key_pressed() -> bool {
    unsafe { GetKeyState(VK_CONTROL) & 0x80 != 0 && GetKeyState(0x50) & 0x80 != 0 }
}
