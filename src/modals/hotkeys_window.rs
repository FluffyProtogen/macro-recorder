use std::{borrow::Cow, cell::RefCell, path::PathBuf, time::SystemTime};

use chrono::{DateTime, Utc};
use egui::{vec2, Align, Align2, Layout, ScrollArea, Window};
use winapi::um::winuser::{GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};

use crate::{
    hotkeys::{start_hotkey_detector, HotkeyMacro},
    keycodes_to_string::key_code_to_string,
};

use super::*;

pub struct HotkeysWindow {
    data: RefCell<HotkeysWindowData>,
}

struct HotkeysWindowData {
    hotkeys: Vec<(Option<Vec<i32>>, Option<PathBuf>, bool)>,
    key_setting_index: Option<usize>,
    key_start_time: Option<DateTime<Utc>>,
}

impl HotkeysWindow {
    pub fn new(hotkeys: Vec<HotkeyMacro>) -> Self {
        let hotkeys = hotkeys
            .iter()
            .map(|hotkey_macro| {
                let HotkeyMacro {
                    hotkeys,
                    path,
                    repeat_if_held,
                } = hotkey_macro.clone();

                (Some(hotkeys), path, repeat_if_held)
            })
            .collect();

        Self {
            data: RefCell::new(HotkeysWindowData {
                hotkeys,
                key_setting_index: None,
                key_start_time: None,
            }),
        }
    }

    fn setup(&self, _recorder: &mut Recorder, drag_bounds: Rect) -> Window {
        Window::new("Hotkeys")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
    }

    fn save(&self, data: &HotkeysWindowData, recorder: &mut Recorder) {
        let mut hotkey_macros = vec![];
        for hotkey_macro in data.hotkeys.clone().into_iter() {
            let (Some(hotkeys), Some(path), repeat_if_held) = hotkey_macro else {
                return;
            };
            hotkey_macros.push(HotkeyMacro {
                hotkeys,
                path: Some(path),
                repeat_if_held,
            });
        }
        recorder.settings.hotkeys = hotkey_macros;
        recorder.modal = recorder.settings.save_with_error_window();
        recorder.hotkey_detector_sender =
            Some(start_hotkey_detector(&mut recorder.settings.hotkeys));
    }
}

fn get_pressed_buttons() -> Vec<i32> {
    (0x01..=0xFE)
        .filter(|key_code| {
            *key_code != VK_SHIFT
                && *key_code != VK_CONTROL
                && *key_code != VK_MENU
                && unsafe { GetAsyncKeyState(*key_code) < 0 }
        })
        .collect()
}

impl ModalWindow for HotkeysWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        _ui: &mut Ui,
        drag_bounds: Rect,
        _frame: &mut eframe::Frame,
    ) {
        let window = self.setup(recorder, drag_bounds);

        window.show(ctx, |ui| {
            let mut data = self.data.borrow_mut();

            ui.add_space(5.0);

            ui.button("Help").on_hover_text(
                "Once you click the button to set a hotkey, \
            you will have 1 second to press and hold all keys until \
            they are registered. \
            Macros launched by a hotkey only play once but can be stopped with Ctrl + Q.",
            );

            ui.add_space(10.0);

            ScrollArea::vertical()
                .max_height(drag_bounds.height() - 200.0)
                .show(ui, |ui| {
                    let mut setting_index = None;
                    let mut added_file = None;
                    let mut remove_index = None;
                    let mut repeat_changed_index = None;
                    for (index, (key_codes, path, repeat_if_held)) in
                        data.hotkeys.iter().enumerate()
                    {
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            ui.add_space(15.0);

                            ui.label(path.as_ref().map_or("Macro Name".to_string(), |path| {
                                let Some(file_name) = path.file_name() else {
                                return "Invalid Macro".into();
                            };
                                file_name
                                    .to_string_lossy()
                                    .to_string()
                                    .split('.')
                                    .next()
                                    .unwrap()
                                    .to_string()
                            }));

                            ui.add_space(5.0);
                            if ui.button("Click to Set").clicked() {
                                if let Some(file) = rfd::FileDialog::new()
                                    .add_filter("fluffy macro", &["floof"])
                                    .add_filter("All files", &["*"])
                                    .pick_file()
                                {
                                    added_file = Some((index, file));
                                }
                            }
                        });

                        ui.add_space(15.0);
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            ui.add_space(15.0);
                            ui.label("Hotkey: ");
                            ui.add_space(5.0);

                            let create_button_text = |keys: &Vec<i32>| {
                                let mut text = String::new();

                                for key in &keys[0..keys.len() - 1] {
                                    text.push_str(&format!("{} + ", key_code_to_string(*key)));
                                }
                                text.push_str(&key_code_to_string(*keys.last().unwrap()));

                                text.into()
                            };

                            let button_text: Cow<str> =
                                if let Some(setting_index) = data.key_setting_index {
                                    if setting_index == index {
                                        let pressed_buttons = get_pressed_buttons();
                                        if pressed_buttons.len() == 0 {
                                            "Press desired button(s)".into()
                                        } else {
                                            create_button_text(&pressed_buttons)
                                        }
                                    } else {
                                        key_codes
                                            .as_ref()
                                            .map_or("Click to set".into(), create_button_text)
                                    }
                                } else {
                                    key_codes
                                        .as_ref()
                                        .map_or("Click to set".into(), create_button_text)
                                };

                            if ui.button(button_text).clicked() {
                                setting_index = Some(index);
                            }
                        });

                        ui.add_space(15.0);
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            ui.add_space(15.0);

                            ui.label("Repeat if button held");
                            ui.add_space(5.0);
                            if ui.checkbox(&mut repeat_if_held.clone(), "").clicked() {
                                repeat_changed_index = Some(index);
                            }
                            ui.add_space(15.0);
                        });

                        ui.add_space(15.0);
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            ui.add_space(15.0);
                            if ui.button("Remove").clicked() {
                                remove_index = Some(index);
                            }
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);
                    }

                    if let Some(index) = setting_index {
                        data.key_setting_index = Some(index);
                    }
                    if let Some((index, file)) = added_file {
                        data.hotkeys[index].1 = Some(file);
                    }
                    if let Some(index) = remove_index {
                        data.hotkeys.remove(index);
                        data.key_setting_index = None;
                        data.key_start_time = None;
                    }
                    if let Some(index) = repeat_changed_index {
                        data.hotkeys[index].2 = !data.hotkeys[index].2;
                    }

                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.add_space(15.0);
                        if ui.button("Add new hotkey").clicked() {
                            data.hotkeys.push((None, None, false));
                        }
                    });

                    if let Some(key_setting_index) = data.key_setting_index {
                        if let Some(time) = data.key_start_time {
                            if (DateTime::from(SystemTime::now()) - time).num_seconds() > 0 {
                                let pressed_buttons = get_pressed_buttons();
                                data.hotkeys[key_setting_index].0 = if pressed_buttons.len() == 0 {
                                    None
                                } else {
                                    Some(pressed_buttons)
                                };
                                data.key_setting_index = None;
                                data.key_start_time = None;
                            }
                        } else {
                            data.key_start_time = Some(DateTime::from(SystemTime::now()));
                        }
                    }
                });

            ui.add_space(15.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                if ui.button("Cancel").clicked() {
                    recorder.modal = None;
                    recorder.hotkey_detector_sender =
                        Some(start_hotkey_detector(&mut recorder.settings.hotkeys));
                }

                ui.add_space(35.0);
                if ui.button("Save").clicked() {
                    self.save(&data, recorder);
                }
            });
        });
    }
}
