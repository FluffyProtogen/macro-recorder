use std::{mem::*, sync::mpsc::sync_channel, thread};
use winapi::{
    ctypes::*,
    shared::minwindef::*,
    um::{processthreadsapi::GetCurrentThreadId, sysinfoapi::GetTickCount, winuser::*},
};

use crate::actions::Action::*;
use crate::actions::MouseActionKind::*;
use crate::actions::{Action, MouseActionButton, Point};
pub fn record_actions(record_mouse_movement: bool) -> Vec<Action> {
    unsafe {
        KEYBOARD_ACTIONS.clear();
        MOUSE_ACTIONS.clear();

        let (keyboard_thread, mouse_thread) = generate_hooks();

        let initial_start_time = GetTickCount();

        while !stop_key_pressed() {}

        PostThreadMessageA(keyboard_thread, WM_QUIT, 0, 0);
        PostThreadMessageA(mouse_thread, WM_QUIT, 0, 0);

        if KEYBOARD_ACTIONS.len() < 2 {
            return vec![];
        }

        // Remove stop combination
        KEYBOARD_ACTIONS.drain((KEYBOARD_ACTIONS.len() - 2)..KEYBOARD_ACTIONS.len());

        if !record_mouse_movement {
            MOUSE_ACTIONS.retain(|(_, w_param)| *w_param != WM_MOUSEMOVE as usize);
        }

        if MOUSE_ACTIONS.len() == 0 && KEYBOARD_ACTIONS.len() == 0 {
            return vec![];
        }

        let start_time = if MOUSE_ACTIONS.len() > 0 {
            if KEYBOARD_ACTIONS.len() > 0 {
                if KEYBOARD_ACTIONS[0].0.time < MOUSE_ACTIONS[0].0.time {
                    KEYBOARD_ACTIONS[0].0.time
                } else {
                    MOUSE_ACTIONS[0].0.time
                }
            } else {
                MOUSE_ACTIONS[0].0.time
            }
        } else {
            KEYBOARD_ACTIONS[0].0.time
        };

        combine_into_action_list(
            &mut KEYBOARD_ACTIONS,
            &mut MOUSE_ACTIONS,
            initial_start_time,
            start_time,
        )
    }
}

fn combine_into_action_list(
    keyboard_actions: &mut Vec<(KBDLLHOOKSTRUCT, WPARAM)>,
    mouse_actions: &mut Vec<(MSLLHOOKSTRUCT, WPARAM)>,
    initial_start_time: u32,
    start_time: u32,
) -> Vec<Action> {
    let mut current_time = start_time;

    let mut keyboard_index = 0;
    let mut mouse_index = 0;

    let mut actions = vec![];

    if (start_time - initial_start_time) > 0 {
        actions.push(Delay((start_time - initial_start_time)));
    }

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
                WM_MOUSEMOVE => Moved(Point {
                    x: mouse_actions[mouse_index].0.pt.x,
                    y: mouse_actions[mouse_index].0.pt.y,
                }),
                WM_MOUSEWHEEL => Wheel(
                    (mouse_actions[mouse_index].0.mouseData as i32) >> 16,
                    Some(Point {
                        x: mouse_actions[mouse_index].0.pt.x,
                        y: mouse_actions[mouse_index].0.pt.y,
                    }),
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
                        point: Some(Point {
                            x: mouse_actions[mouse_index].0.pt.x,
                            y: mouse_actions[mouse_index].0.pt.y,
                        }),
                        button,
                        pressed,
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
                pressed,
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

static mut KEYBOARD_ACTIONS: Vec<(KBDLLHOOKSTRUCT, WPARAM)> = vec![];
static mut MOUSE_ACTIONS: Vec<(MSLLHOOKSTRUCT, WPARAM)> = vec![];

unsafe extern "system" fn keyboard(n_code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let info = *transmute::<LPARAM, PKBDLLHOOKSTRUCT>(l_param);

    KEYBOARD_ACTIONS.push((info, w_param));

    CallNextHookEx(zeroed(), n_code, w_param, l_param)
}

unsafe extern "system" fn mouse(n_code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let info = *transmute::<LPARAM, PMSLLHOOKSTRUCT>(l_param);

    MOUSE_ACTIONS.push((info, w_param));

    CallNextHookEx(zeroed(), n_code, w_param, l_param)
}

pub fn stop_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_CONTROL) < 0 && GetAsyncKeyState(0x51) < 0 }
}
