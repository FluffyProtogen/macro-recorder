extern crate winapi;
use chrono::{DateTime, Utc};
use serde::*;
use serde_json::*;
use std::time::Duration;
use std::time::SystemTime;
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

    unsafe {
        SetCursorPos(action.point.x, action.point.y);
    };

    unsafe { SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32) };
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "POINT")]
struct POINTDef {
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize)]
enum MouseActionKind {
    #[serde(with = "POINTDef")]
    Moved(POINT),
    Button(MouseActionButton),
}

#[derive(Serialize, Deserialize)]
struct MouseActionButton {
    #[serde(with = "POINTDef")]
    point: POINT,
    button: i32,
    pressed: bool,
}

#[derive(Serialize, Deserialize)]
enum Action {
    Delay(u64),
    Mouse(MouseActionKind),
    Keyboard(i32, bool),
}

fn get_key_states() -> Vec<bool> {
    (0..0xFE)
        .map(|key_code| unsafe { GetKeyState(key_code) & 0x80 != 0 })
        .collect()
}

fn get_changed_key_states(
    previous_key_states: &[bool],
    current_key_states: &[bool],
) -> Vec<(usize, bool)> {
    previous_key_states
        .iter()
        .zip(current_key_states)
        .enumerate()
        .filter_map(|(key_code, (previous_state, current_state))| {
            if *previous_state != *current_state {
                Some((key_code, *current_state))
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
    changed_key_states: &[(usize, bool)],
    current_mouse_position: POINT,
    previous_mouse_position: POINT,
    record_mouse_movement: bool,
) {
    if delay != 0 {
        action_list.push(Action::Delay(delay));
    }

    for (key_code, state) in changed_key_states.iter() {
        match *key_code as i32 {
            VK_LBUTTON | VK_RBUTTON | VK_MBUTTON => {
                action_list.push(Action::Mouse(MouseActionKind::Button(MouseActionButton {
                    point: current_mouse_position,
                    button: *key_code as i32,
                    pressed: *state,
                })))
            }
            VK_SHIFT | VK_CONTROL | VK_MENU => (),
            _ => action_list.push(Action::Keyboard(*key_code as i32, *state)),
        }
    }

    if record_mouse_movement
        && (previous_mouse_position.x != current_mouse_position.x
            || previous_mouse_position.y != current_mouse_position.y)
    {
        action_list.push(Action::Mouse(MouseActionKind::Moved(
            current_mouse_position,
        )));
    }
}

fn execute_keyboard_action(key_code: i32, state: bool) {
    let keybd_input = KEYBDINPUT {
        wVk: key_code as u16,
        dwExtraInfo: 0,
        wScan: 0,
        time: 0,
        dwFlags: if state { KEYEVENTF_KEYUP } else { 0 },
    };

    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe { std::mem::transmute_copy(&keybd_input) },
    };

    unsafe {
        SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    };
}

fn play_back_actions(action_list: &[Action]) {
    for action in action_list.iter() {
        match action {
            Action::Keyboard(key_code, state) => execute_keyboard_action(*key_code, *state),
            Action::Delay(delay) => spin_sleep::sleep(Duration::from_micros(*delay)),
            Action::Mouse(action_kind) => match action_kind {
                MouseActionKind::Moved(point) => unsafe {
                    SetCursorPos(point.x, point.y);
                },
                MouseActionKind::Button(action) => execute_mouse_action(action),
            },
        }
    }
}

fn record(record_mouse_movement: bool) -> Vec<Action> {
    let mut previous_key_states = get_key_states();
    let mut previous_mouse_position = get_mouse_position();

    let mut action_list = vec![];

    let mut time_since_last_action = DateTime::<Utc>::from(SystemTime::now());

    for _ in 0..20000 {
        let current_key_states = get_key_states();
        let current_mouse_position = get_mouse_position();

        let changed_key_states = get_changed_key_states(&previous_key_states, &current_key_states);

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

    action_list //
}

fn main() {
    //let action_list = serde_json::from_str::<Vec<_>>(action_list).unwrap();

    let action_list = record(false);
    //println!("{}", action_list.len());
    play_back_actions(&action_list);

    //println!("{}", serde_json::to_string(&action_list).unwrap());
}
