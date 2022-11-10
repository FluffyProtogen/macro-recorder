use crate::recorder::record_actions;

use super::*;

impl App for Recorder {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        Rgba::from_black_alpha(0.0)
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if self.are_any_modals_open() {
            self.right_click_dialog = None;
        }

        PIXELS_PER_POINT.set(ctx.pixels_per_point()).ok();

        if let Some(action) = self.next_play_record_action {
            if action == RecordPlayAction::Play {
                self.hotkey_detector_sender
                    .take()
                    .unwrap()
                    .send(())
                    .unwrap();
                play_back_actions(&self.action_list, &self.settings);
                frame.set_visible(true);
                self.hotkey_detector_sender =
                    Some(start_hotkey_detector(&mut self.settings.hotkeys));
            }
            if action == RecordPlayAction::Record {
                self.hotkey_detector_sender
                    .take()
                    .unwrap()
                    .send(())
                    .unwrap();
                self.action_list = record_actions(&self.settings);
                frame.set_visible(true);
                self.hotkey_detector_sender =
                    Some(start_hotkey_detector(&mut self.settings.hotkeys));
            }
            self.next_play_record_action = None;
        }

        if play_key_pressed() && self.action_list.len() > 0 {
            frame.set_visible(false);
            frame.set_fullscreen(false);
            self.next_play_record_action = Some(RecordPlayAction::Play);
        }

        if self.transparent {
            CentralPanel::default().frame(egui::Frame::default().fill(Color32::TRANSPARENT))
        } else {
            CentralPanel::default()
        }
        .show(ctx, |ui| {
            let moving_let_go_position =
                if self.moving_row && !ui.input().pointer.button_down(PointerButton::Primary) {
                    self.moving_row = false;
                    ui.input().pointer.hover_pos()
                } else {
                    None
                };

            let screen_dimensions = ui.available_size();

            ui.allocate_exact_size(
                vec2(0.0, 60.0),
                Sense {
                    click: true,
                    drag: false,
                    focusable: true,
                },
            );

            egui::Frame::default().show(ui, |ui| {
                ui.set_enabled(!self.are_any_modals_open());
                let row_height = ui.spacing().interact_size.y;
                if !self.transparent {
                    let total_rows = self.action_list.len();
                    ScrollArea::vertical()
                        .enable_scrolling(
                            self.right_click_dialog.is_none() && !self.are_any_modals_open(),
                        )
                        .show_rows(ui, row_height, total_rows, |ui, row_range| {
                            self.add_buttons(
                                ctx,
                                ui,
                                row_range,
                                screen_dimensions,
                                moving_let_go_position,
                            );
                        });

                    self.top_panel(ctx, screen_dimensions, frame);
                    self.side_panel(ctx, screen_dimensions);
                    Self::dividing_lines(ui, screen_dimensions);
                }

                if self.moving_row {
                    self.handle_moving_row(ui, ctx, row_height, screen_dimensions);
                } else {
                    self.handle_main_menu_key_presses(ui, frame, screen_dimensions, ctx);
                }

                if let Some(dialog) = self.right_click_dialog.clone() {
                    dialog.update(self, ctx, ui, screen_dimensions);
                }

                if let Some(modal) = self.modal.clone() {
                    modal.update(
                        self,
                        ctx,
                        ui,
                        Rect::from_x_y_ranges(
                            SIDE_PANEL_WIDTH..=frame.info().window_info.size.x,
                            TOP_PANEL_HEIGHT..=frame.info().window_info.size.y,
                        ),
                        frame,
                    );
                }
            });
        });
        ctx.request_repaint(); // forces egui to render frames at all times, even when not focused (be able to detect play key shortcut even when not focused)
    }
}
