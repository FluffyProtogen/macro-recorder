use crate::modals::action_list_category::{ActionListCategory, ActionListWindow};

use super::*;

impl Recorder {
    pub fn side_panel(&mut self, ctx: &Context, screen_dimensions: Vec2) {
        Area::new("Side Panel")
            .order(Order::Middle)
            .current_pos(Pos2 { x: 0.0, y: 0.0 })
            .show(ctx, |ui| {
                let (_, painter) = ui.allocate_painter(
                    Vec2 {
                        x: 65.0,
                        y: screen_dimensions.y + 25.0,
                    },
                    Sense {
                        click: true,
                        drag: false,
                        focusable: true,
                    },
                );

                painter.rect_filled(
                    Rect::from_x_y_ranges(
                        0.0..=SIDE_PANEL_WIDTH,
                        0.0..=(screen_dimensions.y + 25.0),
                    ),
                    Rounding::none(),
                    Color32::from_rgb(230, 230, 230),
                );

                ui.painter().rect_filled(
                    Rect::from_x_y_ranges(
                        (SIDE_PANEL_WIDTH - 1.5)..=SIDE_PANEL_WIDTH,
                        (TOP_PANEL_HEIGHT - 1.5)..=(screen_dimensions.y + 25.0),
                    ),
                    Rounding::none(),
                    Color32::from_rgb(210, 210, 210),
                );
            });

        Area::new("Side Panel Buttons")
            .current_pos(Pos2 { x: 0.0, y: 0.0 })
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.set_enabled(!self.are_any_modals_open());

                ui.allocate_space(vec2(0.0, 25.0));

                let style = ui
                    .style_mut()
                    .text_styles
                    .get_mut(&crate::gui::TextStyle::Button)
                    .unwrap();

                style.size = 20.0;

                ui.with_layout(Layout::top_down(Align::TOP), |ui| {
                    ui.allocate_space(vec2(65.0, 50.0));

                    if ui.button("Mouse").clicked()
                        || (ui.input().key_pressed(Key::Num1) && !self.are_any_modals_open())
                    {
                        self.create_action_window(
                            Action::Mouse(MouseActionKind::Moved(Point { x: 0, y: 0 })),
                            screen_dimensions,
                            ctx,
                        );
                    }

                    ui.allocate_space(vec2(0.0, 30.0));

                    if ui.button("Key").clicked()
                        || (ui.input().key_pressed(Key::Num2) && !self.are_any_modals_open())
                    {
                        self.create_action_window(
                            Action::Keyboard(0x41, KeyState::Pressed),
                            screen_dimensions,
                            ctx,
                        );
                    }

                    ui.allocate_space(vec2(0.0, 30.0));

                    if ui.button("Wait").clicked()
                        || (ui.input().key_pressed(Key::Num3) && !self.are_any_modals_open())
                    {
                        self.modal = Some(Rc::new(ActionListWindow::new(ActionListCategory::Wait)));
                    }

                    ui.allocate_space(vec2(0.0, 30.0));

                    if ui.button("If").clicked()
                        || (ui.input().key_pressed(Key::Num4) && !self.are_any_modals_open())
                    {
                        self.modal = Some(Rc::new(ActionListWindow::new(ActionListCategory::If)));
                    }
                });
            });
    }
}
