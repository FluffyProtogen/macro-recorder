use std::{cell::RefCell, path::PathBuf};

use crate::{actions::Action, gui::Recorder, modals::ModalWindow};
use eframe::egui::*;

pub struct PlayModifyCommandWindow {
    data: RefCell<PlayModifyCommandWindowData>,
}

struct PlayModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    path: Option<PathBuf>,
    enter_lock: bool,
}

impl PlayModifyCommandWindow {
    pub fn new(creating_command: bool, position: Pos2, path: &PathBuf) -> Self {
        Self {
            data: RefCell::new(PlayModifyCommandWindowData {
                creating_command,
                position: Some(position),
                path: if creating_command {
                    None
                } else {
                    Some(path.clone())
                },
                enter_lock: true,
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        let mut window = Window::new("Play")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds);

        let mut data = self.data.borrow_mut();

        if let Some(position) = data.position {
            window = window.current_pos(Pos2::new(position.x, position.y));
            data.position = None;
        }

        window
    }

    fn save(&self, data: &mut PlayModifyCommandWindowData, recorder: &mut Recorder) {
        let selected_row = recorder.selected_row.unwrap();
        if let Some(path) = data.path.take() {
            recorder.modal = None;
            recorder.action_list()[selected_row] = Action::Play(path);
        }
    }

    fn cancel(&self, data: &PlayModifyCommandWindowData, recorder: &mut Recorder) {
        let selected_row = recorder.selected_row.unwrap();
        recorder.modal = None;
        if data.creating_command {
            recorder.action_list().remove(selected_row);
            recorder.selected_row = None;
        }
    }
}

impl ModalWindow for PlayModifyCommandWindow {
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

            ui.allocate_space(Vec2::new(0.0, 15.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(15.0);
                ui.label(
                    data.path
                        .as_ref()
                        .map_or("None".into(), |path| path.to_string_lossy().to_string()),
                );
            });

            ui.allocate_space(Vec2::new(0.0, 15.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                if ui.button("Select Macro").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("fluffy macro", &["floof"])
                        .add_filter("All files", &["*"])
                        .pick_file()
                    {
                        data.path = Some(path);
                    }
                }
                ui.add_space(35.0);
            });

            ui.add_space(15.0);

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
