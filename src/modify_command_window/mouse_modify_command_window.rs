use std::cell::RefCell;

use super::ModifyCommandWindow;
use crate::{
    actions::{self, Action, MouseActionButton, MouseActionButtonState, MouseActionKind, Point},
    gui::Recorder,
};
use eframe::egui::*;
use std::fmt::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use winapi::um::winuser::*;

pub struct MouseModifyCommandWindow {
    data: RefCell<MouseModifyCommandWindowData>,
}

struct MouseModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    combo_box_type: MouseComboBoxType,
    mouse_position_checkbox_state: bool,
    mouse_position_text_edit_text: (String, String),
    scroll_text_edit_text: String,
    f2_previously_pressed: bool,
    f3_previously_pressed: bool,
    window_visible: bool,
}

impl MouseModifyCommandWindow {
    pub fn new(
        creating_command: bool,
        position: Pos2,
        mouse_action_kind: &MouseActionKind,
    ) -> Self {
        let mouse_position = match mouse_action_kind {
            MouseActionKind::Button(button) => button.point,
            MouseActionKind::Moved(point) => Some(*point),
            MouseActionKind::Wheel(_, point) => *point,
        };

        let mouse_position_text_edit_text = if let Some(position) = mouse_position {
            (position.x.to_string(), position.y.to_string())
        } else {
            (String::new(), String::new())
        };

        let scroll_text_edit_text =
            if let actions::MouseActionKind::Wheel(amount, _) = mouse_action_kind {
                (amount / 120).to_string()
            } else {
                String::new()
            };

        Self {
            data: RefCell::new(MouseModifyCommandWindowData {
                creating_command,
                position: Some(position),
                combo_box_type: MouseComboBoxType::from(mouse_action_kind),
                mouse_position_checkbox_state: mouse_position.is_some(),
                mouse_position_text_edit_text,
                scroll_text_edit_text,
                f2_previously_pressed: capture_mouse_position_key_pressed(),
                f3_previously_pressed: minimize_window_key_pressed(),
                window_visible: true,
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        let mut window = Window::new("Mouse")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds);

        let mut data = self.data.borrow_mut();

        if let Some(position) = &data.position {
            window = window.current_pos(Pos2::new(position.x, position.y));
            data.position = None;
        }

        window
    }
}

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy)]
enum MouseComboBoxType {
    LeftClick,
    RightClick,
    MiddleClick,
    Move,
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
    MiddleDown,
    MiddleUp,
    Wheel,
}

impl From<&MouseActionKind> for MouseComboBoxType {
    fn from(item: &MouseActionKind) -> Self {
        match item {
            MouseActionKind::Moved(_) => Self::Move,
            MouseActionKind::Wheel(_, _) => Self::Wheel,
            MouseActionKind::Button(button) => match button.button {
                VK_LBUTTON => match button.state {
                    MouseActionButtonState::Pressed => Self::LeftDown,
                    MouseActionButtonState::Released => Self::LeftUp,
                    MouseActionButtonState::Clicked => Self::LeftClick,
                    _ => panic!(),
                },
                VK_RBUTTON => match button.state {
                    MouseActionButtonState::Pressed => Self::RightDown,
                    MouseActionButtonState::Released => Self::RightUp,
                    MouseActionButtonState::Clicked => Self::RightClick,
                    _ => panic!(),
                },
                VK_MBUTTON => match button.state {
                    MouseActionButtonState::Pressed => Self::MiddleDown,
                    MouseActionButtonState::Released => Self::MiddleUp,
                    MouseActionButtonState::Clicked => Self::MiddleClick,
                    _ => panic!(),
                },
                _ => panic!(),
            },
        }
    }
}

impl Display for MouseComboBoxType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match self {
                Self::LeftClick => "Left Click",
                Self::RightClick => "Right Click",
                Self::MiddleClick => "Middle Click",
                Self::Move => "Move",
                Self::LeftDown => "Left Button Down",
                Self::LeftUp => "Left Button Up",
                Self::RightDown => "Right Button Down",
                Self::RightUp => "Right Button Up",
                Self::MiddleDown => "Middle Button Down",
                Self::MiddleUp => "Middle Button Up",
                Self::Wheel => "Wheel",
            }
        )
    }
}

impl ModifyCommandWindow for MouseModifyCommandWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        drag_bounds: Rect,
        frame: &mut eframe::Frame,
    ) {
        let window = self.setup(drag_bounds);

        window.show(ctx, |ui| {
            let data = &mut self.data.borrow_mut();

            if ui.input().key_pressed(Key::Enter) {
                self.save(data, recorder);
            }
            if ui.input().key_pressed(Key::Escape) {
                self.cancel(data, recorder);
            }

            if capture_mouse_position_key_pressed() {
                if !data.f2_previously_pressed {
                    let mut point = unsafe { std::mem::zeroed() };
                    unsafe {
                        GetCursorPos(&mut point);
                    };

                    data.mouse_position_text_edit_text = (point.x.to_string(), point.y.to_string());
                    data.window_visible = true;
                    frame.set_visible(true);
                    data.f2_previously_pressed = true;
                }
            } else {
                data.f2_previously_pressed = false;
            }

            if minimize_window_key_pressed() {
                if !data.f3_previously_pressed {
                    data.f3_previously_pressed = true;

                    data.window_visible = !data.window_visible;
                    frame.set_visible(data.window_visible);
                }
            } else {
                data.f3_previously_pressed = false;
            }

            ui.allocate_space(vec2(0.0, 10.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                let selected = &mut data.combo_box_type;

                ui.label("Event Type: ");
                ui.add_space(10.0);
                ComboBox::new("Mouse Combo Box", "")
                    .selected_text(format!("{}", selected))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for combo_box_type in MouseComboBoxType::iter() {
                            let text = format!("{}", combo_box_type);

                            ui.selectable_value(selected, combo_box_type, text);

                            ui.allocate_space(vec2(0.0, 3.5));
                        }
                    });
            });

            if data.combo_box_type != MouseComboBoxType::Move {
                ui.allocate_space(vec2(0.0, 25.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(45.0);

                    ui.label("Use a mouse position?").on_hover_text(
                        "If false, the mouse action will execute at the current mouse position.",
                    );

                    ui.add_space(10.0);

                    let mouse_position_checked = &mut data.mouse_position_checkbox_state;
                    ui.checkbox(mouse_position_checked, "");
                    data.mouse_position_checkbox_state = *mouse_position_checked;
                });
            }

            if data.mouse_position_checkbox_state || data.combo_box_type == MouseComboBoxType::Move
            {
                ui.allocate_space(vec2(0.0, 25.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(35.0);

                    ui.label("X: ");

                    TextEdit::singleline(&mut data.mouse_position_text_edit_text.0)
                        .desired_width(50.0)
                        .ui(ui);

                    ui.add_space(15.0);

                    ui.label("Y: ");

                    TextEdit::singleline(&mut data.mouse_position_text_edit_text.1)
                        .desired_width(50.0)
                        .ui(ui);
                });

                ui.allocate_space(vec2(0.0, 25.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    let mut point = unsafe { std::mem::zeroed() };
                    unsafe {
                        GetCursorPos(&mut point);
                    };

                    ui.add_space(15.0);

                    ui.label(format!(
                        "Current mouse position: ({}, {})",
                        point.x, point.y
                    ));
                });

                ui.add_space(35.0);
                ui.label("Press F2 to capture the current mouse position.");
                ui.add_space(15.0);
                ui.label("Press F3 to hide the window.");
            }

            if data.combo_box_type == MouseComboBoxType::Wheel {
                ui.allocate_space(vec2(0.0, 25.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(55.0);

                    ui.label("Scroll wheel: ");

                    TextEdit::singleline(&mut data.scroll_text_edit_text)
                        .desired_width(25.0)
                        .ui(ui);
                });
            }

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                if ui.button("Cancel").clicked() {
                    self.cancel(data, recorder);
                }
                ui.add_space(35.0);
                if ui.button("Save").clicked() {
                    self.save(data, recorder);
                }
            });
        });
    }
}

impl MouseModifyCommandWindow {
    fn save(&self, data: &MouseModifyCommandWindowData, recorder: &mut Recorder) {
        match data.combo_box_type {
            MouseComboBoxType::Wheel => {
                if let Ok(mut scroll) = data.scroll_text_edit_text.parse() {
                    scroll *= 120;

                    if data.mouse_position_checkbox_state {
                        if let (Ok(x), Ok(y)) = (
                            data.mouse_position_text_edit_text.0.parse(),
                            data.mouse_position_text_edit_text.1.parse(),
                        ) {
                            recorder.modify_command_window = None;
                            recorder.action_list[recorder.selected_row.unwrap()] =
                                Action::Mouse(MouseActionKind::Wheel(scroll, Some(Point { x, y })));
                        }
                    } else {
                        recorder.modify_command_window = None;
                        recorder.action_list[recorder.selected_row.unwrap()] =
                            Action::Mouse(MouseActionKind::Wheel(scroll, None));
                    }
                }
            }
            MouseComboBoxType::Move => {
                if let (Ok(x), Ok(y)) = (
                    data.mouse_position_text_edit_text.0.parse(),
                    data.mouse_position_text_edit_text.1.parse(),
                ) {
                    recorder.modify_command_window = None;
                    recorder.action_list[recorder.selected_row.unwrap()] =
                        Action::Mouse(MouseActionKind::Moved(Point { x, y }));
                }
            }
            _ => {
                let button = match data.combo_box_type {
                    MouseComboBoxType::LeftClick
                    | MouseComboBoxType::LeftDown
                    | MouseComboBoxType::LeftUp => VK_LBUTTON,
                    MouseComboBoxType::RightClick
                    | MouseComboBoxType::RightDown
                    | MouseComboBoxType::RightUp => VK_RBUTTON,
                    MouseComboBoxType::MiddleClick
                    | MouseComboBoxType::MiddleDown
                    | MouseComboBoxType::MiddleUp => VK_MBUTTON,
                    _ => unreachable!(),
                };

                let state = match data.combo_box_type {
                    MouseComboBoxType::LeftClick
                    | MouseComboBoxType::RightClick
                    | MouseComboBoxType::MiddleClick => MouseActionButtonState::Clicked,
                    MouseComboBoxType::LeftDown
                    | MouseComboBoxType::RightDown
                    | MouseComboBoxType::MiddleDown => MouseActionButtonState::Pressed,
                    MouseComboBoxType::LeftUp
                    | MouseComboBoxType::RightUp
                    | MouseComboBoxType::MiddleUp => MouseActionButtonState::Released,
                    _ => unreachable!(),
                };

                if data.mouse_position_checkbox_state {
                    if let (Ok(x), Ok(y)) = (
                        data.mouse_position_text_edit_text.0.parse(),
                        data.mouse_position_text_edit_text.1.parse(),
                    ) {
                        recorder.modify_command_window = None;
                        recorder.action_list[recorder.selected_row.unwrap()] =
                            Action::Mouse(MouseActionKind::Button(MouseActionButton {
                                point: Some(Point { x, y }),
                                button,
                                state,
                            }));
                    }
                } else {
                    recorder.modify_command_window = None;
                    recorder.action_list[recorder.selected_row.unwrap()] =
                        Action::Mouse(MouseActionKind::Button(MouseActionButton {
                            point: None,
                            button,
                            state,
                        }));
                }
            }
        }
    }

    fn cancel(&self, data: &MouseModifyCommandWindowData, recorder: &mut Recorder) {
        recorder.modify_command_window = None;
        if data.creating_command {
            recorder.action_list.remove(recorder.selected_row.unwrap());
            recorder.selected_row = None;
        }
    }
}

pub fn capture_mouse_position_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_F2) < 0 }
}

pub fn minimize_window_key_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_F3) < 0 }
}
