use std::cell::RefCell;

use crate::{actions::Action, gui::Recorder};
use eframe::{egui::*, *};
pub trait ModifyCommandWindow {
    fn update(&self, recorder: &mut Recorder, ctx: &Context, ui: &mut Ui, screen_dimensions: Rect) {
    }
}

impl Action {
    pub fn get_modify_command_window(
        &self,
        creating_command: bool,
        position: Vec2,
    ) -> Box<dyn ModifyCommandWindow> {
        match self {
            Self::Mouse(_) => Box::new(MouseModifyCommandWindow {
                data: RefCell::new(MouseModifyCommandWindowData {
                    creating_command,
                    position: Some(position),
                }),
            }),
            Self::Delay(_) => Box::new(DelayModifyCommandWindow::new(creating_command, position)),
            Self::Keyboard(_, _) => panic!(),
        }
    }
}

struct DelayModifyCommandWindow {
    data: RefCell<DelayModifyCommandWindowData>,
}

struct DelayModifyCommandWindowData {
    creating_command: bool,
    position: Option<Vec2>,
    text_edit_text: String,
}

impl DelayModifyCommandWindow {
    fn new(creating_command: bool, position: Vec2) -> Self {
        Self {
            data: RefCell::new(DelayModifyCommandWindowData {
                creating_command,
                position: Some(position),
                text_edit_text: String::new(),
            }),
        }
    }
}

impl ModifyCommandWindow for DelayModifyCommandWindow {
    fn update(&self, recorder: &mut Recorder, ctx: &Context, ui: &mut Ui, drag_bounds: Rect) {
        let mut window = Window::new("Delay")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds);
        {
            let mut data = self.data.borrow_mut();

            if let Some(position) = &data.position {
                window = window.current_pos(Pos2::new(position.x, position.y));
                data.position = None;
            }
        }

        window.show(ctx, |ui| {
            let data = &mut self.data.borrow_mut();

            let duration_area = TextEdit::singleline(&mut data.text_edit_text).desired_width(50.0);

            ui.allocate_space(Vec2::new(0.0, 25.0));

            Grid::new("Mouse Window Text Area").show(ui, |ui| {
                ui.allocate_space(Vec2::new(25.0, 0.0));
                duration_area.ui(ui);
                ui.allocate_space(Vec2::new(25.0, 0.0));
            });

            ui.allocate_space(Vec2::new(0.0, 25.0));

            Grid::new("Mouse Window Layout").show(ui, |ui| {
                if ui.button("Cancel").clicked() {
                    recorder.modify_command_window = None;
                    if data.creating_command {
                        recorder.action_list.remove(recorder.selected_row.unwrap());
                        recorder.selected_row = None;
                    }
                }

                if ui.button("Save").clicked() {
                    if let Ok(delay) = data.text_edit_text.parse::<u64>() {
                        recorder.modify_command_window = None;
                        recorder.action_list[recorder.selected_row.unwrap()] = Action::Delay(delay);
                    }
                }
            });

            //println!("{}", self.data.borrow().text_edit_text);
            //self.data.borrow_mut().text_edit_text = duration;
        });
    }
}

struct MouseModifyCommandWindow {
    data: RefCell<MouseModifyCommandWindowData>,
}

struct MouseModifyCommandWindowData {
    creating_command: bool,
    position: Option<Vec2>,
}

impl ModifyCommandWindow for MouseModifyCommandWindow {
    fn update(&self, recorder: &mut Recorder, ctx: &Context, ui: &mut Ui, drag_bounds: Rect) {
        let mut window = Window::new("Mouse Command")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds);
        {
            let mut data = self.data.borrow_mut();

            if let Some(position) = &data.position {
                window = window.current_pos(Pos2::new(position.x, position.y));
                data.position = None;
            }
        }

        window.show(ctx, |ui| {});
    }
}
