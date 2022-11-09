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
    pub path: PathBuf,
    pub repeat_if_held: bool,
}

pub fn start_hotkey_detector(hotkeys: Vec<HotkeyMacro>) -> Sender<()> {
    let (sender, receiver) = channel();

    let mut loaded_hotkeys = vec![];

    for hotkey in hotkeys {
        let action_list = load_from_file(&hotkey.path).unwrap();

        loaded_hotkeys.push(LoadedHotkeyMacro {
            action_list,
            key_combination: hotkey.hotkeys,
            repeat_if_held: hotkey.repeat_if_held,
        });
    }

    thread::spawn(|| hotkey_detector(loaded_hotkeys, receiver));

    sender
}

//AFTER CLONING THE SETTINGS, MAKE THE AMOUNT OF TIMES IT REPEATS 1 CUZ IT CAN BE REPEATED MORE TIMES FROM JUST HOLDING THE KEY IF ENABLED
//NEED TO HANDLE INVALID MACRO TOO!
fn hotkey_detector(hotkeys: Vec<LoadedHotkeyMacro>, receiver: Receiver<()>) {
    let mut playing_states = vec![false; hotkeys.len()];

    let sendersReceivers = (0..hotkeys.len())
        .map(|index| {
            let (executor_sender, detector_receiver) = channel();
            let (detector_sender, executor_receiver) = channel();

            let action_list = hotkeys[index].action_list.clone();

            thread::spawn(move || action_executor(action_list, executor_sender, executor_receiver));

            (detector_sender, detector_receiver)
        })
        .collect::<Vec<_>>();

    loop {
        if receiver.try_recv().is_ok() {
            for (sender, _) in sendersReceivers {
                sender.send(ExecutorMessage::Quit).unwrap();
            }
            break;
        }

        for (index, state) in playing_states.clone().iter().enumerate() {
            let hotkey_macro = &hotkeys[index];
            if !state && hotkeys_pressed(&hotkey_macro.key_combination) {
                playing_states[index] = true;
                sendersReceivers[index]
                    .0
                    .send(ExecutorMessage::Play)
                    .unwrap();
            }
        }

        for (index, (_, receiver)) in sendersReceivers.iter().enumerate() {
            if receiver.try_recv().is_ok() {
                playing_states[index] = false;
            }
        }
    }
}

fn action_executor(actions: Vec<Action>, sender: Sender<()>, receiver: Receiver<ExecutorMessage>) {
    loop {
        if let Ok(message) = receiver.try_recv() {
            match message {
                ExecutorMessage::Quit => {
                    println!("killed");
                    break;
                }
                ExecutorMessage::Play => {
                    play_back_actions(&actions, &Default::default());
                    sender.send(()).unwrap();
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

enum ExecutorMessage {
    Play,
    Quit,
}
