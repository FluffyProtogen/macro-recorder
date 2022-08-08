//#![windows_subsystem = "windows"]

use eframe::{egui::*, *};

use std::rc::Rc;

use crate::{
    actions::Action,
    modify_command_window::ModifyCommandWindow,
    play_back_actions, play_key_pressed,
    right_click_dialog::ActionRightClickDialog,
    settings::{self, Settings},
    settings_window::SettingsWindow,
    warning_window::{DefaultErrorWindow, SettingsNotFoundErrorWindow, WarningWindow},
};

pub struct Recorder {
    pub selected_row: Option<usize>,
    pub action_list: Vec<Action>,
    pub right_click_dialog: Option<Rc<ActionRightClickDialog>>,
    pub modify_command_window: Option<Rc<Box<dyn ModifyCommandWindow>>>,
    next_play_record_action: Option<RecordPlayAction>,
    pub settings_window: Option<Rc<SettingsWindow>>,
    pub settings: Settings,
    pub warning_window: Option<Rc<Box<dyn WarningWindow>>>,
}

const TOP_PANEL_HEIGHT: f32 = 65.0;

const SIDE_PANEL_WIDTH: f32 = 65.0;

#[derive(PartialEq, Eq)]
enum RecordPlayAction {
    Play,
    Record,
}

impl Recorder {
    pub fn new(cc: &CreationContext<'_>, action_list: Vec<Action>) -> Self {
        use crate::gui::FontFamily::*;
        use crate::gui::TextStyle::*;
        use crate::settings::*;
        cc.egui_ctx.set_visuals(Visuals::light());

        let mut style = (*cc.egui_ctx.style()).clone();

        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (
                TextStyle::Name("Heading2".into()),
                FontId::new(25.0, Proportional),
            ),
            (
                TextStyle::Name("Context".into()),
                FontId::new(23.0, Proportional),
            ),
            (Body, FontId::new(18.0, Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(18.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ]
        .into();

        style.spacing.item_spacing = vec2(0.0, 0.0);

        cc.egui_ctx.set_style(style);

        let settings = settings::load_settings();

        let warning_window = if settings.is_err() {
            let create_settings_file_result = settings::create_settings_file();

            match create_settings_file_result {
                Ok(()) => Some(Rc::new(DefaultErrorWindow::new(
                    "Settings Not Found".into(),
                    vec![
                        "Settings file not found.".into(),
                        format!(
                            "Settings file created at {}\\{}",
                            std::env::current_dir().unwrap().to_str().unwrap(),
                            settings::SETTINGS_FILE_NAME
                        ),
                    ],
                ))),
                Err(error) => Some(Rc::new(DefaultErrorWindow::new(
                    "Settings Error".into(),
                    vec![
                        "Settings file not found.".into(),
                        "Error attempting to create settings file:".into(),
                        error.to_string(),
                    ],
                ))),
            }
        } else {
            None
        };

        Self {
            selected_row: None,
            action_list,
            right_click_dialog: None,
            modify_command_window: None,
            next_play_record_action: None,
            settings_window: None,
            settings: settings.unwrap_or(Default::default()),
            warning_window,
        }
    }

    fn top_panel(&mut self, ctx: &Context, screen_dimensions: Vec2, frame: &mut eframe::Frame) {
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

                ui.allocate_space(vec2(0.0, 25.0));

                let style = ui
                    .style_mut()
                    .text_styles
                    .get_mut(&crate::gui::TextStyle::Button)
                    .unwrap();

                style.size = 35.0;

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.allocate_space(vec2(65.0, 0.0));
                    let record_button = Button::new("Record");
                    let record_response = record_button.ui(ui);

                    ui.allocate_space(vec2(50.0, 0.0));

                    let play_button = Button::new("Play");
                    let play_response = play_button.ui(ui);

                    ui.allocate_space(vec2(50.0, 0.0));

                    let settings_button = Button::new("Settings");
                    let settings_response = settings_button.ui(ui);

                    if play_response.clicked() {
                        self.right_click_dialog = None;
                        self.next_play_record_action = Some(RecordPlayAction::Play);
                        frame.set_visible(false);
                        frame.set_fullscreen(false);
                    }

                    if record_response.clicked() {
                        self.right_click_dialog = None;
                        self.next_play_record_action = Some(RecordPlayAction::Record);
                        frame.set_visible(false);
                        frame.set_fullscreen(false);
                    }

                    if settings_response.clicked() {
                        self.right_click_dialog = None;
                        self.settings_window =
                            Some(Rc::new(SettingsWindow::new(self.settings.clone())));
                    }
                });
            });
    }

    fn side_panel(ctx: &Context, screen_dimensions: Vec2) {
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
    }

    fn dividing_lines(ui: &mut Ui, screen_dimensions: Vec2) {
        let step = (screen_dimensions.x - SIDE_PANEL_WIDTH) / 3.0;

        for i in 1..=2 {
            ui.painter().rect_filled(
                Rect::from_x_y_ranges(
                    (step * i as f32 + -0.75 + SIDE_PANEL_WIDTH)
                        ..=(step * i as f32 + 0.75 + SIDE_PANEL_WIDTH),
                    TOP_PANEL_HEIGHT..=(screen_dimensions.y + 25.0),
                ),
                Rounding::none(),
                Color32::from_rgb(210, 210, 210),
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
        spacing: f32,
    ) {
        let info = self.action_list.get(row).unwrap().get_grid_formatted();

        for (count, info) in info.iter().enumerate() {
            let step = (screen_dimensions.x - SIDE_PANEL_WIDTH) / 3.0;

            Area::new(format!("area{}{}", count, row))
                .interactable(false)
                .order(Order::Background)
                .fixed_pos(pos2(
                    97.0 + count as f32 * step,
                    start_pos + spacing * ((row - row_range.start) as f32 + 1.0),
                ))
                .show(ctx, |ui| {
                    ui.label(info);
                });
        }
    }

    fn add_buttons(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        row_range: std::ops::Range<usize>,
        screen_dimensions: Vec2,
    ) {
        let mut start_pos = 0.0;
        for row in row_range.clone().into_iter() {
            let button_color = if let Some(selected_row) = self.selected_row {
                if selected_row == row {
                    Color32::from_rgba_premultiplied(189, 231, 255, 255)
                } else {
                    Color32::from_rgba_premultiplied(0, 0, 0, if row % 2 == 0 { 10 } else { 30 })
                }
            } else {
                Color32::from_rgba_premultiplied(0, 0, 0, if row % 2 == 0 { 10 } else { 30 })
            };

            let button = Button::new(" ".repeat(1000)).wrap(false).fill(button_color);

            let response = button.ui(ui);

            if response.clicked() {
                self.selected_row = Some(row);
                self.right_click_dialog = None;
            }

            if response.secondary_clicked() {
                self.selected_row = Some(row);

                self.right_click_dialog = Some(Rc::new(
                    self.action_list[row].get_right_click_dialog(response.hover_pos().unwrap()),
                ));
            }

            if response.double_clicked() {
                self.modify_command_window = Some(Rc::new(
                    self.action_list[self.selected_row.unwrap()]
                        .get_modify_command_window(false, response.hover_pos().unwrap()),
                ));
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
                response.rect.height(),
            );
        }
    }

    fn handle_main_menu_key_presses(&mut self, ui: &mut Ui) {
        if self.modify_command_window.is_some() {
            return;
        }

        if ui.input().key_pressed(Key::Delete) || ui.input().key_pressed(Key::Backspace) {
            if let Some(selected_row) = self.selected_row {
                self.action_list.remove(selected_row);
                self.selected_row = None;
                self.right_click_dialog = None;
            }
        }

        if ui.input().key_pressed(Key::Escape) {
            self.right_click_dialog = None;
            self.selected_row = None;
        }
    }

    fn are_any_modals_open(&self) -> bool {
        self.modify_command_window.is_some()
            || self.settings_window.is_some()
            || self.warning_window.is_some()
    }
}

impl App for Recorder {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
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

        if play_key_pressed() {
            frame.set_visible(false);
            frame.set_fullscreen(false);
            self.next_play_record_action = Some(RecordPlayAction::Play);
        }

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
                ui.set_enabled(!self.are_any_modals_open());

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
                Self::side_panel(ctx, screen_dimensions);
                Self::dividing_lines(ui, screen_dimensions);

                self.handle_main_menu_key_presses(ui);

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
                    );
                }
            });
        });
    }
}
