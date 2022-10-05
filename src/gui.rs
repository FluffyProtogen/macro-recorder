use eframe::{egui::*, *};
use once_cell::sync::OnceCell;

use std::{path::*, rc::Rc};

pub static PIXELS_PER_POINT: OnceCell<f32> = OnceCell::new();

use crate::{
    actions::{Action, KeyState, MouseActionKind, Point},
    load_from_file,
    modify_command_window::ModifyCommandWindow,
    play_back_actions, play_key_pressed,
    right_click_dialog::ActionRightClickDialog,
    save_macro,
    settings::{self, Settings},
    settings_window::SettingsWindow,
    warning_window::{DefaultErrorWindow, RecordConfirmationWindow, WarningWindow},
};

pub struct Recorder {
    pub selected_row: Option<usize>,
    pub action_list: Vec<Action>,
    pub right_click_dialog: Option<Rc<ActionRightClickDialog>>,
    pub modify_command_window: Option<Rc<Box<dyn ModifyCommandWindow>>>,
    pub next_play_record_action: Option<RecordPlayAction>,
    pub settings_window: Option<Rc<SettingsWindow>>,
    pub settings: Settings,
    pub warning_window: Option<Rc<Box<dyn WarningWindow>>>,
    pub current_macro_path: Option<PathBuf>,
    pub transparent: bool,
}

const TOP_PANEL_HEIGHT: f32 = 65.0;

const SIDE_PANEL_WIDTH: f32 = 65.0;

#[derive(PartialEq, Eq)]
pub enum RecordPlayAction {
    Play,
    Record,
}

impl Recorder {
    pub fn new(cc: &CreationContext<'_>, action_list: Vec<Action>) -> Self {
        use crate::gui::FontFamily::*;
        use crate::gui::TextStyle::*;
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
            current_macro_path: None,
            transparent: false,
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

                        ui.allocate_space(vec2(30.0, 0.0));
                    }

                    if ui.button("Record").clicked() {
                        self.right_click_dialog = None;

                        if self.action_list.len() > 0 {
                            self.warning_window = Some(Rc::new(RecordConfirmationWindow::new()));
                        } else {
                            self.next_play_record_action = Some(RecordPlayAction::Record);
                            frame.set_visible(false);
                            frame.set_fullscreen(false);
                        }
                    }

                    ui.allocate_space(vec2(30.0, 0.0));

                    if ui.button("Settings").clicked() {
                        self.right_click_dialog = None;
                        self.settings_window =
                            Some(Rc::new(SettingsWindow::new(self.settings.clone())));
                    }

                    ui.allocate_space(vec2(30.0, 0.0));

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
                                    self.warning_window = Some(Rc::new(DefaultErrorWindow::new(
                                        "Load Error".into(),
                                        vec!["Error loading macro:".into(), error.to_string()],
                                    )))
                                }
                            }
                        }
                    }

                    if self.action_list.len() > 0 {
                        ui.allocate_space(vec2(30.0, 0.0));

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
                        }

                        ui.allocate_space(vec2(30.0, 0.0));

                        if ui.button("Save As").clicked() {
                            self.right_click_dialog = None;
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("fluffy macro", &["floof"])
                                .save_file()
                            {
                                self.try_save(path, frame);
                            }
                        }
                    }
                });
            });
    }

    fn side_panel(&mut self, ctx: &Context, screen_dimensions: Vec2) {
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

                    if ui.button("Mouse").clicked() {
                        self.create_action_window(
                            Action::Mouse(MouseActionKind::Moved(Point { x: 0, y: 0 })),
                            screen_dimensions,
                        );
                    }

                    ui.allocate_space(vec2(0.0, 30.0));

                    if ui.button("Key").clicked() {
                        self.create_action_window(
                            Action::Keyboard(0x41, KeyState::Pressed),
                            screen_dimensions,
                        );
                    }

                    ui.allocate_space(vec2(0.0, 30.0));

                    ui.menu_button("Wait", |ui| {
                        ui.add_space(3.0);
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            ui.add_space(5.0);
                            if ui.button("Delay").clicked() {
                                self.create_action_window(Action::Delay(0), screen_dimensions);
                            }
                        });

                        ui.allocate_space(vec2(0.0, 10.0));
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            ui.add_space(5.0);
                            if ui.button("Wait for image").clicked() {
                                self.create_action_window(Action::WaitForImage, screen_dimensions);
                            }
                        });
                        ui.add_space(3.0);
                    });
                });
            });
    }

    fn create_action_window(&mut self, action: Action, screen_dimensions: Vec2) {
        self.modify_command_window = Some(Rc::new(action.get_modify_command_window(
            true,
            pos2(
                screen_dimensions.x / 2.0 - SIDE_PANEL_WIDTH,
                screen_dimensions.y / 2.0 - TOP_PANEL_HEIGHT,
            ),
        )));

        self.create_action(action);
    }

    fn create_action(&mut self, action: Action) {
        if let Some(row) = self.selected_row {
            self.action_list.insert(row + 1, action);
            self.selected_row = Some(row + 1);
        } else {
            self.action_list.push(action);
            self.selected_row = Some(self.action_list.len() - 1);
        }
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

    fn handle_main_menu_key_presses(
        &mut self,
        ui: &mut Ui,
        frame: &mut eframe::Frame,
        screen_dimensions: Vec2,
    ) {
        if self.are_any_modals_open() {
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

        if ui.input().modifiers.ctrl && ui.input().key_pressed(Key::S) && self.action_list.len() > 0
        {
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
        }

        if let (true, Some(row)) = (ui.input().key_pressed(Key::Enter), self.selected_row) {
            self.modify_command_window = Some(Rc::new(
                self.action_list[self.selected_row.unwrap()].get_modify_command_window(
                    false,
                    pos2(
                        screen_dimensions.x / 2.0 - SIDE_PANEL_WIDTH,
                        screen_dimensions.y / 2.0 - TOP_PANEL_HEIGHT,
                    ),
                ),
            ));
        }
    }

    fn are_any_modals_open(&self) -> bool {
        self.modify_command_window.is_some()
            || self.settings_window.is_some()
            || self.warning_window.is_some()
    }

    fn try_save(&mut self, path: PathBuf, frame: &mut eframe::Frame) {
        let save_result = save_macro(&path, &self.action_list);

        if let Err(error) = save_result {
            self.warning_window = Some(Rc::new(DefaultErrorWindow::new(
                "Save Error".into(),
                vec!["Error saving macro:".into(), error.to_string()],
            )))
        } else {
            self.current_macro_path = Some(path);
            self.update_title(frame);
        }
    }

    fn update_title(&self, frame: &mut eframe::Frame) {
        frame.set_window_title(&format!(
            "Fluffy Macro Recorder - {}",
            self.current_macro_path.as_ref().unwrap().to_str().unwrap()
        ));
    }
}

impl App for Recorder {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        Rgba::from_black_alpha(0.0)
    }

    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
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
                self.handle_main_menu_key_presses(ui, frame, screen_dimensions);

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
