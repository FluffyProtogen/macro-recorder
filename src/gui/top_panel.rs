use crate::modals::{
    hotkeys_window::HotkeysWindow,
    settings_window::SettingsWindow,
    warning_window::{DefaultErrorWindow, RecordConfirmationWindow},
};

use super::*;

impl Recorder {
    pub fn top_panel(&mut self, ctx: &Context, screen_dimensions: Vec2, frame: &mut eframe::Frame) {
        Area::new("Top Panel")
            .current_pos(Pos2 { x: 0.0, y: 0.0 })
            .order(Order::Middle)
            .show(ctx, |ui| {
                let (_, painter) = ui.allocate_painter(
                    Vec2 {
                        x: (screen_dimensions.x + 25.0),
                        y: TOP_PANEL_HEIGHT,
                    },
                    Sense {
                        click: true,
                        drag: false,
                        focusable: true,
                    },
                );

                painter.rect_filled(
                    Rect::from_x_y_ranges(
                        0.0..=(screen_dimensions.x + 25.0),
                        0.0..=TOP_PANEL_HEIGHT,
                    ),
                    Rounding::none(),
                    Color32::from_rgb(230, 230, 230),
                );

                ui.painter().rect_filled(
                    Rect::from_x_y_ranges(
                        (TOP_PANEL_HEIGHT - 2.0)..=(screen_dimensions.x + 25.0),
                        63.5..=TOP_PANEL_HEIGHT,
                    ),
                    Rounding::none(),
                    Color32::from_rgb(210, 210, 210),
                );
            });

        Area::new("Top Panel Buttons")
            .current_pos(Pos2 { x: 0.0, y: 0.0 })
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.set_enabled(!self.are_any_modals_open());

                ui.allocate_space(vec2(0.0, 20.0));

                let style = ui
                    .style_mut()
                    .text_styles
                    .get_mut(&crate::gui::TextStyle::Button)
                    .unwrap();

                style.size = 30.0;

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.allocate_space(vec2(65.0, 0.0));

                    if self.action_list.len() > 0 {
                        if ui.button("Play").clicked() {
                            self.right_click_dialog = None;
                            self.next_play_record_action = Some(RecordPlayAction::Play);
                            frame.set_visible(false);
                            frame.set_fullscreen(false);
                        }

                        ui.allocate_space(vec2(20.0, 0.0));
                    }

                    if ui.button("Record").clicked() {
                        self.right_click_dialog = None;

                        if self.action_list.len() > 0 {
                            self.modal = Some(RecordConfirmationWindow::new());
                        } else {
                            self.next_play_record_action = Some(RecordPlayAction::Record);
                            frame.set_visible(false);
                            frame.set_fullscreen(false);
                        }
                    }

                    ui.allocate_space(vec2(20.0, 0.0));

                    if ui.button("Settings").clicked() {
                        self.right_click_dialog = None;
                        self.modal = Some(Rc::new(SettingsWindow::new(self.settings.clone())));
                    }

                    ui.allocate_space(vec2(20.0, 0.0));

                    if ui.button("Open").clicked() {
                        self.right_click_dialog = None;
                        let path = rfd::FileDialog::new()
                            .add_filter("fluffy macro", &["floof"])
                            .add_filter("All files", &["*"])
                            .pick_file();
                        if let Some(path) = path {
                            let load_result = load_from_file(&path);

                            match load_result {
                                Ok(result) => {
                                    self.current_macro_path = Some(path);
                                    self.action_list = result;
                                    self.update_title(frame);
                                }
                                Err(error) => {
                                    self.modal = Some(DefaultErrorWindow::new(
                                        "Load Error".into(),
                                        vec!["Error loading macro:".into(), error.to_string()],
                                    ))
                                }
                            }
                        }
                    }

                    if self.action_list.len() > 0 {
                        ui.allocate_space(vec2(20.0, 0.0));

                        if ui.button("Save").clicked() {
                            self.right_click_dialog = None;
                            if let Some(path) = &self.current_macro_path.clone() {
                                self.try_save(path.clone(), frame);
                            } else {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("fluffy macro", &["floof"])
                                    .save_file()
                                {
                                    self.try_save(path, frame);
                                }
                            }
                            self.hotkey_detector_sender
                                .take()
                                .unwrap()
                                .send(())
                                .unwrap();
                            self.hotkey_detector_sender =
                                Some(start_hotkey_detector(self.settings.hotkeys.clone()));
                        }

                        ui.allocate_space(vec2(20.0, 0.0));

                        if ui.button("Save As").clicked() {
                            self.right_click_dialog = None;
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("fluffy macro", &["floof"])
                                .save_file()
                            {
                                self.try_save(path, frame);
                                self.hotkey_detector_sender
                                    .take()
                                    .unwrap()
                                    .send(())
                                    .unwrap();
                                self.hotkey_detector_sender =
                                    Some(start_hotkey_detector(self.settings.hotkeys.clone()));
                            }
                        }
                    }
                    ui.allocate_space(vec2(20.0, 0.0));

                    if ui.button("Hotkeys").clicked() {
                        self.hotkey_detector_sender
                            .take()
                            .unwrap()
                            .send(())
                            .unwrap();
                        self.modal =
                            Some(Rc::new(HotkeysWindow::new(self.settings.hotkeys.clone())));
                    }
                });
            });
    }
}
