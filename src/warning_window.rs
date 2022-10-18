use std::rc::Rc;

use egui::*;

use crate::{
    gui::{RecordPlayAction, Recorder},
    ModalWindow,
};

pub struct DefaultErrorWindow {
    title: String,
    lines: Vec<String>,
}

impl DefaultErrorWindow {
    pub fn new(title: String, lines: Vec<String>) -> Rc<dyn ModalWindow> {
        Rc::new(Self { title, lines })
    }
}

impl ModalWindow for DefaultErrorWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        _ui: &mut Ui,
        drag_bounds: Rect,
        _frame: &mut eframe::Frame,
    ) {
        let window = Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0));

        window.show(ctx, |ui| {
            for line in self.lines.iter() {
                ui.allocate_space(vec2(0.0, 25.0));

                ui.label(line);
            }

            ui.allocate_space(vec2(0.0, 25.0));

            if ui.button("Ok").clicked() {
                recorder.modal = None;
            }
        });
    }
}

pub struct RecordConfirmationWindow;

impl ModalWindow for RecordConfirmationWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        _ui: &mut Ui,
        drag_bounds: Rect,
        frame: &mut eframe::Frame,
    ) {
        let window = Window::new("Confirmation")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0));

        window.show(ctx, |ui| {
            ui.allocate_space(vec2(0.0, 25.0));

            ui.label(
                "Recording will replace the existing macro.\nMake sure to save if you haven't!",
            );

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                if ui.button("Cancel").clicked() {
                    recorder.modal = None;
                }

                ui.add_space(25.0);

                if ui.button("Record").clicked() {
                    recorder.modal = None;
                    recorder.next_play_record_action = Some(RecordPlayAction::Record);
                    frame.set_visible(false);
                    frame.set_fullscreen(false);
                }
            });
        });
    }
}

impl RecordConfirmationWindow {
    pub fn new() -> Rc<dyn ModalWindow> {
        Rc::new(Self {})
    }
}
