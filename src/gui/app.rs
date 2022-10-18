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

        if let Some(action) = &self.next_play_record_action {
            if *action == RecordPlayAction::Play {
                play_back_actions(&self.action_list, &self.settings);
                frame.set_visible(true);
            }
            if *action == RecordPlayAction::Record {
                self.action_list =
                    crate::recorder::record_actions(self.settings.record_mouse_movement);
                frame.set_visible(true);
                self.next_play_record_action = None;
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
            let screen_dimensions = ui.available_size();

            ui.allocate_exact_size(
                Vec2 { x: 0.0, y: 60.0 },
                Sense {
                    click: true,
                    drag: false,
                    focusable: true,
                },
            );

            egui::Frame::default().show(ui, |ui| {
                ui.set_enabled(!self.are_any_modals_open());

                if !self.transparent {
                    let row_height = ui.spacing().interact_size.y;

                    let total_rows = self.action_list.len();
                    ScrollArea::vertical()
                        .enable_scrolling(
                            self.right_click_dialog.is_none() && !self.are_any_modals_open(),
                        )
                        .show_rows(ui, row_height, total_rows, |ui, row_range| {
                            self.add_buttons(ctx, ui, row_range, screen_dimensions);
                        });

                    self.top_panel(ctx, screen_dimensions, frame);
                    self.side_panel(ctx, screen_dimensions);
                    Self::dividing_lines(ui, screen_dimensions);
                }
                self.handle_main_menu_key_presses(ui, frame, screen_dimensions, ctx);

                if let Some(dialog) = &self.right_click_dialog.clone() {
                    dialog.update(self, ctx, ui, screen_dimensions);
                }

                if let Some(window) = &self.modify_command_window.clone() {
                    window.update(
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

                if let Some(settings_window) = &self.settings_window.clone() {
                    settings_window.update(
                        self,
                        ctx,
                        ui,
                        Rect::from_x_y_ranges(
                            SIDE_PANEL_WIDTH..=frame.info().window_info.size.x,
                            TOP_PANEL_HEIGHT..=frame.info().window_info.size.y,
                        ),
                    );
                }

                if let Some(warning_window) = &self.warning_window.clone() {
                    warning_window.update(
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
        ctx.request_repaint();
    }
}
