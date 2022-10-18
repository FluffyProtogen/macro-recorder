use std::cell::RefCell;

use crate::{actions::Action, gui::Recorder, ModalWindow};
use eframe::egui::*;

pub struct DelayModifyCommandWindow {
    data: RefCell<DelayModifyCommandWindowData>,
}

struct DelayModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    text_edit_text: String,
    enter_lock: bool,
}

impl DelayModifyCommandWindow {
    pub fn new(creating_command: bool, position: Pos2, delay: u32) -> Self {
        Self {
            data: RefCell::new(DelayModifyCommandWindowData {
                creating_command,
                position: Some(position),
                text_edit_text: delay.to_string(),
                enter_lock: true,
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

    fn save(&self, data: &DelayModifyCommandWindowData, recorder: &mut Recorder) {
        if let Ok(delay) = data.text_edit_text.parse::<u32>() {
            recorder.modal = None;
            recorder.action_list[recorder.selected_row.unwrap()] = Action::Delay(delay);
        }
    }

    fn cancel(&self, data: &DelayModifyCommandWindowData, recorder: &mut Recorder) {
        recorder.modal = None;
        if data.creating_command {
            recorder.action_list.remove(recorder.selected_row.unwrap());
            recorder.selected_row = None;
        }
    }
}

impl ModalWindow for DelayModifyCommandWindow {
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

            if ui.input().key_down(Key::Enter) {
                if !data.enter_lock {
                    self.save(data, recorder);
                }
            } else {
                data.enter_lock = false;
            }
            if ui.input().key_pressed(Key::Escape) {
                self.cancel(data, recorder);
            }

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
