use std::cell::RefCell;

use crate::{actions::Action, gui::Recorder};
use eframe::egui::*;

use super::ModifyCommandWindow;

pub struct DelayModifyCommandWindow {
    data: RefCell<DelayModifyCommandWindowData>,
}

struct DelayModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    text_edit_text: String,
}

impl DelayModifyCommandWindow {
    pub fn new(creating_command: bool, position: Pos2, delay: u32) -> Self {
        Self {
            data: RefCell::new(DelayModifyCommandWindowData {
                creating_command,
                position: Some(position),
                text_edit_text: delay.to_string(),
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        let mut window = Window::new("Delay")
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

impl ModifyCommandWindow for DelayModifyCommandWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        _ui: &mut Ui,
        drag_bounds: Rect,
        _frame: &mut eframe::Frame,
    ) {
        let window = self.setup(drag_bounds);

        window.show(ctx, |ui| {
            let data = &mut self.data.borrow_mut();

            ui.allocate_space(Vec2::new(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                let duration_area =
                    TextEdit::singleline(&mut data.text_edit_text).desired_width(150.0);

                ui.add_space(35.0);
                duration_area.ui(ui);
                ui.add_space(15.0);
                ui.label("milliseconds");
                ui.add_space(35.0);
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
                    if let Ok(delay) = data.text_edit_text.parse::<u32>() {
                        recorder.modify_command_window = None;
                        recorder.action_list[recorder.selected_row.unwrap()] = Action::Delay(delay);
                    }
                }
            });
        });
    }
}
