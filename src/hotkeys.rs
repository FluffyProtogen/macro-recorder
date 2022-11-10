use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    path::PathBuf,
    rc::Rc,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};
use winapi::um::winuser::GetAsyncKeyState;

use crate::{
    actions::Action,
    load_from_file,
    modals::{warning_window::DefaultErrorWindow, ModalWindow},
    play_back_actions,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct HotkeyMacro {
    pub hotkeys: Vec<i32>,
    pub path: Option<PathBuf>,
    pub repeat_if_held: bool,
}

pub fn start_hotkey_detector(hotkeys: &mut Vec<HotkeyMacro>) -> Sender<()> {
    let (sender, receiver) = channel();

    let mut loaded_hotkeys = vec![];

    for (index, hotkey) in hotkeys.clone().into_iter().enumerate() {
        let Some(path) = &hotkey.path else {
            continue;
        };

        let Ok(action_list) = load_from_file(path) else {
            hotkeys[index].path = None;
            continue;
        };

        loaded_hotkeys.push(LoadedHotkeyMacro {
            action_list,
            key_combination: hotkey.hotkeys,
            repeat_if_held: hotkey.repeat_if_held,
        });
    }

    thread::spawn(|| hotkey_detector(loaded_hotkeys, receiver));

    sender
}

//NEED TO HANDLE INVALID MACRO TOO!
fn hotkey_detector(hotkeys: Vec<LoadedHotkeyMacro>, receiver: Receiver<()>) {
    let senders = hotkeys
        .into_iter()
        .map(|hotkey_macro| {
            let (detector_sender, executor_receiver) = channel();

            thread::spawn(move || action_executor(hotkey_macro, executor_receiver));

            detector_sender
        })
        .collect::<Vec<_>>();

    loop {
        if receiver.try_recv().is_ok() {
            for sender in senders {
                sender.send(()).unwrap();
            }
            return;
        }
    }
}

fn action_executor(hotkey_macro: LoadedHotkeyMacro, receiver: Receiver<()>) {
    loop {
        if receiver.try_recv().is_ok() {
            return;
        }

        if hotkeys_pressed(&hotkey_macro.key_combination) {
            play_back_actions(&hotkey_macro.action_list, &Default::default());
        }

        if !hotkey_macro.repeat_if_held {
            loop {
                if receiver.try_recv().is_ok() {
                    return;
                }
                if !hotkeys_pressed(&hotkey_macro.key_combination) {
                    break;
                }
            }
        }
    }
}

fn hotkeys_pressed(hotkeys: &[i32]) -> bool {
    for hotkey in hotkeys {
        if !unsafe { GetAsyncKeyState(*hotkey) < 0 } {
            return false;
        }
    }
    true
}

struct LoadedHotkeyMacro {
    action_list: Vec<Action>,
    key_combination: Vec<i32>,
    repeat_if_held: bool,
}
