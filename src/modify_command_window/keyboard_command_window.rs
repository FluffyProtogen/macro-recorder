use std::cell::RefCell;

use crate::{
    actions::{self, Action, KeyState},
    gui::Recorder,
    keycodes_to_string::{key_code_to_string, ALLOWED_KEYBOARD_KEYS},
};
use eframe::{egui::*, *};

use super::ModifyCommandWindow;

pub struct KeyboardModifyCommandWindow {
    data: RefCell<KeyboardModifyCommandWindowData>,
}

struct KeyboardModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    key_state: KeyState,
    key_code: Option<i32>,
    key_code_text_edit_text: String,
}

impl KeyboardModifyCommandWindow {
    pub fn new(creating_command: bool, position: Pos2, key_code: i32, key_state: KeyState) -> Self {
        Self {
            data: RefCell::new(KeyboardModifyCommandWindowData {
                creating_command,
                position: Some(position),
                key_state,
                key_code: Some(key_code),
                key_code_text_edit_text: key_code.to_string(),
            }),
        }
    }

    fn setup(&self, recorder: &mut Recorder, drag_bounds: Rect) -> Window {
        let mut window = Window::new("Keyboard")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds);

        let mut data = self.data.borrow_mut();

        if let Some(position) = data.position {
            window = window.current_pos(position);
            data.position = None;
        }

        window
    }
}

impl ModifyCommandWindow for KeyboardModifyCommandWindow {
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

            ui.allocate_space(vec2(0.0, 25.0));
            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                let selected = &mut data.key_state;
                ui.add_space(35.0);
                ui.label("Event Type: ");
                ui.add_space(10.0);
                ComboBox::new("Key Event Combo Box", "")
                    .selected_text(format!("{:?}", selected))
                    .width(80.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(selected, KeyState::Down, "Down");
                        ui.allocate_space(vec2(0.0, 3.5));
                        ui.selectable_value(selected, KeyState::Up, "Up");
                        ui.allocate_space(vec2(0.0, 3.5));
                        ui.selectable_value(selected, KeyState::Pressed, "Pressed");
                    });
            });

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                ui.label("Key: ");
                ui.add_space(10.0);
                ComboBox::new("Key Combo Box", "")
                    .selected_text(if let Some(key_code) = data.key_code {
                        key_code_to_string(key_code)
                    } else {
                        "Invalid Key Code".to_string()
                    })
                    .width(140.0)
                    .show_ui(ui, |ui| {
                        for key_code in ALLOWED_KEYBOARD_KEYS.iter() {
                            if ui
                                .selectable_value(
                                    &mut data.key_code,
                                    Some(*key_code),
                                    key_code_to_string(*key_code),
                                )
                                .clicked()
                            {
                                data.key_code_text_edit_text = key_code.to_string();
                            }
                            ui.allocate_space(vec2(0.0, 3.5));
                        }
                    });
            });

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(15.0);
                ui.label("Key Code: ");
                ui.add_space(15.0);
                TextEdit::singleline(&mut data.key_code_text_edit_text)
                    .desired_width(50.0)
                    .ui(ui);

                data.key_code = data.key_code_text_edit_text.parse().ok();
            });

            ui.allocate_space(Vec2::new(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                if ui.button("Cancel").clicked() {
                    recorder.modify_command_window = None;
                    if data.creating_command {
                        recorder.action_list.remove(recorder.selected_row.unwrap());
                        recorder.selected_row = None;
                    }
                }
                ui.add_space(35.0);
                if ui.button("Save").clicked() {
                    if let Some(key_code) = data.key_code {
                        recorder.modify_command_window = None;
                        recorder.action_list[recorder.selected_row.unwrap()] =
                            Action::Keyboard(key_code, data.key_state);
                    }
                }
            });
        });
    }
}
