use egui::*;

use crate::{gui::Recorder, recorder, settings};

pub trait WarningWindow {
    fn update(&self, recorder: &mut Recorder, ctx: &Context, ui: &mut Ui, drag_bounds: Rect) {}
}

pub struct SettingsNotFoundErrorWindow {}

impl SettingsNotFoundErrorWindow {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct DefaultErrorWindow {
    title: String,
    lines: Vec<String>,
}

impl DefaultErrorWindow {
    pub fn new(title: String, lines: Vec<String>) -> Box<dyn WarningWindow> {
        Box::new(Self { title, lines })
    }
}

impl WarningWindow for DefaultErrorWindow {
    fn update(&self, recorder: &mut Recorder, ctx: &Context, ui: &mut Ui, drag_bounds: Rect) {
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
                recorder.warning_window = None;
            }
        });
    }
}

impl WarningWindow for SettingsNotFoundErrorWindow {
    fn update(&self, recorder: &mut Recorder, ctx: &Context, ui: &mut Ui, drag_bounds: Rect) {
        let window = Window::new("Settings Not Found")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0));

        window.show(ctx, |ui| {
            ui.allocate_space(vec2(0.0, 25.0));

            ui.label("Settings file not found.");

            ui.allocate_space(vec2(0.0, 25.0));

            let area_width = ui
                .label(format!(
                    "Settings file created at {}\\{}",
                    std::env::current_dir().unwrap().to_str().unwrap(),
                    settings::SETTINGS_FILE_NAME
                ))
                .rect
                .width();

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(area_width / 2.0);
                if ui.button("Ok").clicked() {
                    recorder.warning_window = None;
                }
            });
        });
    }
}
