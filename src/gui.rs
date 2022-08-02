//#![windows_subsystem = "windows"]

use std::ops::RangeInclusive;

use eframe::{egui::*, *};

use crate::actions::Action;

#[derive(Default)]
pub struct Recorder {
    action_list: Vec<Action>,
}

const TOP_PANEL_HEIGHT: f32 = 65.0;

const SIDE_PANEL_WIDTH: f32 = 65.0;

impl Recorder {
    pub fn new(cc: &CreationContext<'_>, action_list: Vec<Action>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());
        Self { action_list }
    }

    fn top_panel(ctx: &Context, screen_dimensions: Vec2) {
        Area::new("Top Panel")
            .current_pos(Pos2 { x: 0.0, y: 0.0 })
            .order(Order::Foreground)
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
                    Color32::from_rgb(30, 30, 30),
                );

                ui.painter().rect_filled(
                    Rect::from_x_y_ranges(
                        (TOP_PANEL_HEIGHT - 2.0)..=(screen_dimensions.x + 25.0),
                        63.0..=TOP_PANEL_HEIGHT,
                    ),
                    Rounding::none(),
                    Color32::from_rgb(100, 100, 100),
                )
            });
    }

    fn side_panel(ctx: &Context, screen_dimensions: Vec2) {
        Area::new("Side Panel")
            .order(Order::Foreground)
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
                    Color32::from_rgb(30, 30, 30),
                );

                ui.painter().rect_filled(
                    Rect::from_x_y_ranges(
                        (SIDE_PANEL_WIDTH - 2.0)..=SIDE_PANEL_WIDTH,
                        (TOP_PANEL_HEIGHT - 2.0)..=(screen_dimensions.y + 25.0),
                    ),
                    Rounding::none(),
                    Color32::from_rgb(100, 100, 100),
                )
            });
    }

    fn dividing_lines(ui: &mut Ui, screen_dimensions: Vec2) {
        let step = (screen_dimensions.x - SIDE_PANEL_WIDTH) / 3.0;

        for i in 1..=2 {
            ui.painter().rect_filled(
                Rect::from_x_y_ranges(
                    (step * i as f32 + -1.0 + SIDE_PANEL_WIDTH)
                        ..=(step * i as f32 + 1.0 + SIDE_PANEL_WIDTH),
                    TOP_PANEL_HEIGHT..=(screen_dimensions.y + 25.0),
                ),
                Rounding::none(),
                Color32::from_rgb(100, 100, 100),
            );
        }
    }

    fn add_row_label(
        &self,
        ctx: &Context,
        row: usize,
        screen_dimensions: Vec2,
        start_pos: f32,
        row_range: std::ops::Range<usize>,
    ) {
        let info = self.action_list.get(row).unwrap().get_grid_formatted();

        for (count, info) in info.iter().enumerate() {
            let step = (screen_dimensions.x - SIDE_PANEL_WIDTH) / 3.0;

            Area::new(format!("area{}{}", count, row))
                .interactable(false)
                .order(Order::Middle)
                .fixed_pos(pos2(
                    97.0 + count as f32 * step,
                    start_pos + 21.0 * ((row - row_range.start) as f32 + 1.0),
                ))
                .show(ctx, |ui| {
                    ui.label(info);
                });
        }
    }
}

impl App for Recorder {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
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
                let row_height = ui.spacing().interact_size.y;
                let total_rows = self.action_list.len();
                ScrollArea::vertical().show_rows(ui, row_height, total_rows, |ui, row_range| {
                    let mut start_pos = 0.0;
                    for row in row_range.clone().into_iter() {
                        let button = Button::new(" ".repeat(1000)).wrap(false).fill(
                            Color32::from_rgba_premultiplied(
                                0,
                                0,
                                0,
                                if row % 2 == 0 { 35 } else { 85 },
                            ),
                        );

                        let response = button.ui(ui);
                        if response.clicked() {
                            println!("{}", row);
                        }

                        if row == row_range.clone().start {
                            start_pos = response.rect.top() - response.rect.height();
                        }

                        self.add_row_label(
                            ctx,
                            row,
                            screen_dimensions,
                            start_pos,
                            row_range.clone(),
                        );
                    }
                });
                Self::top_panel(ctx, screen_dimensions);
                Self::side_panel(ctx, screen_dimensions);
                Self::dividing_lines(ui, screen_dimensions);
            });
        });
    }
}
