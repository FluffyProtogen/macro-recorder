use chrono::{DateTime, Utc};
use std::{mem::*, sync::mpsc::sync_channel, thread, time::SystemTime};
use winapi::{
    ctypes::*,
    shared::{minwindef::*, windef::POINT},
    um::{processthreadsapi::GetCurrentThreadId, sysinfoapi::GetTickCount, winuser::*},
};

use crate::actions::{Action::*, MouseActionButtonState};
use crate::actions::{KeyState, MouseActionKind::*};
use crate::{
    actions::{Action, MouseActionButton, MousePointKind, Point},
    settings::Settings,
};
pub fn record_actions(settings: &Settings) -> Vec<Action> {
    unsafe {
        let mut initial_position = zeroed();

        GetCursorPos(&mut initial_position);

        KEYBOARD_ACTIONS.clear();
        MOUSE_ACTIONS.clear();

        let (keyboard_thread, mouse_thread) = generate_hooks();

        let initial_start_time = DateTime::<Utc>::from(SystemTime::now()).timestamp_millis();

        while !stop_key_pressed() {}

        PostThreadMessageA(keyboard_thread, WM_QUIT, 0, 0);
        PostThreadMessageA(mouse_thread, WM_QUIT, 0, 0);

        if KEYBOARD_ACTIONS.len() < 2 {
            return vec![];
        }

        // Remove stop combination
        KEYBOARD_ACTIONS.drain((KEYBOARD_ACTIONS.len() - 2)..KEYBOARD_ACTIONS.len());

        if !settings.record_mouse_movement {
            MOUSE_ACTIONS.retain(|(_, w_param, _)| *w_param != WM_MOUSEMOVE as usize);
        }

        if MOUSE_ACTIONS.len() == 0 && KEYBOARD_ACTIONS.len() == 0 {
            return vec![];
        }

        let mut kb_actions = KEYBOARD_ACTIONS
            .iter()
            .map(|(kb, w_param, time)| {
                (
                    KBDLLHOOKSTRUCT {
                        time: (DateTime::<Utc>::from(*time).timestamp_millis() - initial_start_time)
                            as u32,
                        ..*kb
                    },
                    *w_param,
                )
            })
            .collect::<Vec<_>>();

        let mut ms_actions = MOUSE_ACTIONS
            .iter()
            .map(|(ms, w_param, time)| {
                (
                    MSLLHOOKSTRUCT {
                        time: (DateTime::<Utc>::from(*time).timestamp_millis() - initial_start_time)
                            as u32,
                        ..*ms
                    },
                    *w_param,
                )
            })
            .collect::<Vec<_>>();

        let start_time = if ms_actions.len() > 0 {
            if kb_actions.len() > 0 {
                if kb_actions[0].0.time < ms_actions[0].0.time {
                    kb_actions[0].0.time
                } else {
                    ms_actions[0].0.time
                }
            } else {
                ms_actions[0].0.time
            }
        } else {
            kb_actions[0].0.time
        };

        combine_into_action_list(
            &mut kb_actions,
            &mut ms_actions,
            0,
            start_time,
            settings,
            initial_position,
        )
    }
}

fn combine_into_action_list(
    keyboard_actions: &mut Vec<(KBDLLHOOKSTRUCT, WPARAM)>,
    mouse_actions: &mut Vec<(MSLLHOOKSTRUCT, WPARAM)>,
    initial_start_time: u32,
    start_time: u32,
    settings: &Settings,
    initial_position: POINT,
) -> Vec<Action> {
    let mut current_time = start_time;

    let mut keyboard_index = 0;
    let mut mouse_index = 0;

    let mut actions = vec![];

    if (start_time - initial_start_time) > 0 {
        actions.push(Delay(start_time - initial_start_time));
    }

    let mut previous_position = initial_position;

    let mut calculate_mouse_pos = |point: POINT| {
        if settings.record_mouse_offsets {
            let point_kind = MousePointKind::By(Point {
                x: point.x - previous_position.x,
                y: point.y - previous_position.y,
            });

            previous_position = point;

            point_kind
        } else {
            MousePointKind::To(Point {
                x: point.x,
                y: point.y,
            })
        }
    };

    loop {
        let next_keyboard_time = if keyboard_index < keyboard_actions.len() {
            keyboard_actions[keyboard_index].0.time as u64
        } else {
            u64::MAX
        };

        let next_mouse_time = if mouse_index < mouse_actions.len() {
            mouse_actions[mouse_index].0.time as u64
        } else {
            u64::MAX
        };

        if next_keyboard_time > next_mouse_time {
            let action_kind = match mouse_actions[mouse_index].1 as u32 {
                WM_MOUSEMOVE => Moved(calculate_mouse_pos(mouse_actions[mouse_index].0.pt)),
                WM_MOUSEWHEEL => Wheel(
                    (mouse_actions[mouse_index].0.mouseData as i32) >> 16,
                    Some(calculate_mouse_pos(mouse_actions[mouse_index].0.pt)),
                ),
                _ => {
                    let button = match mouse_actions[mouse_index].1 as u32 {
                        WM_LBUTTONDOWN | WM_LBUTTONUP => 1,
                        WM_RBUTTONDOWN | WM_RBUTTONUP => 2,
                        WM_MBUTTONDOWN | WM_MBUTTONUP => 4,
                        _ => mouse_actions[mouse_index].1 as i32,
                    };

                    let pressed = match mouse_actions[mouse_index].1 as u32 {
                        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => true,
                        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => false,
                        _ => false,
                    };

                    let action = MouseActionButton {
                        point: Some(calculate_mouse_pos(mouse_actions[mouse_index].0.pt)),
                        button,
                        state: if pressed {
                            MouseActionButtonState::Pressed
                        } else {
                            MouseActionButtonState::Released
                        },
                    };

                    Button(action)
                }
            };

            if mouse_actions[mouse_index].0.time - current_time != 0 {
                actions.push(Delay(mouse_actions[mouse_index].0.time - current_time));
            }

            current_time = mouse_actions[mouse_index].0.time;

            actions.push(Mouse(action_kind));
            mouse_index += 1;
        } else {
            if keyboard_actions[keyboard_index].0.time - current_time != 0 {
                actions.push(Delay(
                    keyboard_actions[keyboard_index].0.time - current_time,
                ));
            }

            let pressed = match keyboard_actions[keyboard_index].1 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => true,
                WM_KEYUP | WM_SYSKEYUP => false,
                _ => panic!(),
            };

            actions.push(Keyboard(
                keyboard_actions[keyboard_index].0.vkCode as i32,
                if pressed {
                    KeyState::Down
                } else {
                    KeyState::Up
                },
            ));

            current_time = keyboard_actions[keyboard_index].0.time;

            keyboard_index += 1;
        }

        if mouse_index == mouse_actions.len() && keyboard_index == keyboard_actions.len() {
            break actions;
        }
    }
}

unsafe fn generate_hooks() -> (u32, u32) {
    let (keyboard_sender, keyboard_receiver) = sync_channel(0);

    thread::spawn(move || {
        let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard), std::ptr::null_mut(), 0);

        let mut msg = zeroed();

        keyboard_sender.send(GetCurrentThreadId()).unwrap();

        while GetMessageA(&mut msg, zeroed(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        UnhookWindowsHookEx(hook);
    });

    let (mouse_sender, mouse_receiver) = sync_channel(0);

    thread::spawn(move || {
        let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(mouse), zeroed(), 0);

        let mut msg = zeroed();

        mouse_sender.send(GetCurrentThreadId()).unwrap();

        while GetMessageA(&mut msg, zeroed(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        UnhookWindowsHookEx(hook);
    });

    (
        keyboard_receiver.recv().unwrap(),
        mouse_receiver.recv().unwrap(),
    )
}

static mut KEYBOARD_ACTIONS: Vec<(KBDLLHOOKSTRUCT, WPARAM, SystemTime)> = vec![];
static mut MOUSE_ACTIONS: Vec<(MSLLHOOKSTRUCT, WPARAM, SystemTime)> = vec![];

unsafe extern "system" fn keyboard(n_code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let info = *transmute::<LPARAM, PKBDLLHOOKSTRUCT>(l_param);

    KEYBOARD_ACTIONS.push((info, w_param, SystemTime::now()));

    CallNextHookEx(zeroed(), n_code, w_param, l_param)
}

unsafe extern "system" fn mouse(n_code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let info = *transmute::<LPARAM, PMSLLHOOKSTRUCT>(l_param);

    MOUSE_ACTIONS.push((info, w_param, SystemTime::now()));

    CallNextHookEx(zeroed(), n_code, w_param, l_param)
}

pub fn stop_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_CONTROL) < 0 && GetAsyncKeyState(0x51) < 0 }
}
