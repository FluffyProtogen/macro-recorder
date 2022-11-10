use std::cell::RefCell;

use egui::*;

use crate::{gui::Recorder, settings, Settings};

use super::{warning_window::DefaultErrorWindow, ModalWindow};

pub struct SettingsWindow {
    data: RefCell<SettingsWindowData>,
}

impl SettingsWindow {
    pub fn new(settings: Settings) -> Self {
        let replay_textedit_text = settings.repeat_times.to_string();

        Self {
            data: RefCell::new(SettingsWindowData {
                temp_settings: settings,
                replay_textedit_text,
            }),
        }
    }

    fn setup(&self, _recorder: &mut Recorder, drag_bounds: Rect) -> Window {
        Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
    }
}

struct SettingsWindowData {
    temp_settings: Settings,
    replay_textedit_text: String,
}

impl ModalWindow for SettingsWindow {
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

            ui.checkbox(
                &mut data.temp_settings.record_mouse_movement,
                "Record mouse movement",
            );

            ui.allocate_space(vec2(0.0, 25.0));

            ui.checkbox(
                &mut data.temp_settings.record_mouse_offsets,
                "Record mouse offsets instead of position",
            );

            ui.allocate_space(vec2(0.0, 25.0));

            ui.label(format!(
                "Playback speed: {}x",
                data.temp_settings.playback_speed
            ));

            Slider::new(&mut data.temp_settings.playback_speed, 0.5..=5.0).ui(ui);

            ui.allocate_space(vec2(0.0, 25.0));

            ui.checkbox(
                &mut data.temp_settings.ignore_delays,
                "Ignore delays completely",
            );

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                let repeat_area =
                    TextEdit::singleline(&mut data.replay_textedit_text).desired_width(75.0);

                repeat_area.ui(ui);
                ui.add_space(25.0);
                ui.label("Number of times to repeat. Put 0 for infinite repeats.");
            });

            ui.allocate_space(vec2(0.0, 25.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                if ui.button("Cancel").clicked() {
                    recorder.modal = None;
                }
                ui.add_space(35.0);
                if ui.button("Save").clicked() {
                    if let Ok(repeats) = data.replay_textedit_text.parse::<u32>() {
                        data.temp_settings.repeat_times = repeats;
                        recorder.settings = data.temp_settings.clone();
                        recorder.modal = data.temp_settings.save_with_error_window();
                    }
                }
            });
        });
    }
}
