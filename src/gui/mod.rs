use eframe::{egui::*, *};
use once_cell::sync::OnceCell;

use std::{path::*, rc::Rc, sync::mpsc::Sender};

pub static PIXELS_PER_POINT: OnceCell<f32> = OnceCell::new();
pub const ROW_LABEL_X_OFFSET: f32 = 97.0;

pub mod app;
pub mod side_panel;
pub mod top_panel;

use crate::{
    actions::{Action, KeyState, MouseActionKind, Point},
    hotkeys::start_hotkey_detector,
    load_from_file,
    modals::{warning_window::DefaultErrorWindow, ModalWindow},
    play_back_actions, play_key_pressed,
    right_click_dialog::ActionRightClickDialog,
    save_macro,
    settings::{self, Settings},
};

pub struct Recorder {
    pub selected_row: Option<usize>,
    pub action_list: Vec<Action>,
    pub right_click_dialog: Option<Rc<ActionRightClickDialog>>,
    pub next_play_record_action: Option<RecordPlayAction>,
    pub settings: Settings,
    pub current_macro_path: Option<PathBuf>,
    pub transparent: bool,
    pub scroll_to_me_row: Option<usize>,
    pub modal: Option<Rc<dyn ModalWindow>>,
    pub moving_row: bool,
    pub hotkey_detector_sender: Option<Sender<()>>,
}

const TOP_PANEL_HEIGHT: f32 = 65.0;

const SIDE_PANEL_WIDTH: f32 = 65.0;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RecordPlayAction {
    Play,
    Record,
}

impl Recorder {
    pub fn new(cc: &CreationContext, action_list: Vec<Action>) -> Self {
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
                Ok(()) => Some(DefaultErrorWindow::new(
                    "Settings Not Found".into(),
                    vec![
                        "Settings file not found.".into(),
                        format!(
                            "Settings file created at {}\\{}",
                            std::env::current_dir().unwrap().to_str().unwrap(),
                            settings::SETTINGS_FILE_NAME
                        ),
                    ],
                )),
                Err(error) => Some(DefaultErrorWindow::new(
                    "Settings Error".into(),
                    vec![
                        "Settings file not found.".into(),
                        "Error attempting to create settings file:".into(),
                        error.to_string(),
                    ],
                )),
            }
        } else {
            None
        };

        let hotkey_detector_sender = Some(start_hotkey_detector(
            settings
                .as_ref()
                .map_or(vec![], |settings| settings.hotkeys.clone()),
        ));

        Self {
            selected_row: None,
            action_list,
            right_click_dialog: None,
            next_play_record_action: None,
            settings: settings.unwrap_or(Default::default()),
            current_macro_path: None,
            transparent: false,
            scroll_to_me_row: None,
            modal: warning_window,
            moving_row: false,
            hotkey_detector_sender,
        }
    }

    pub fn create_action_window(&mut self, action: Action, screen_dimensions: Vec2, ctx: &Context) {
        self.modal = action.get_modify_command_window(
            true,
            pos2(
                screen_dimensions.x / 2.0 - SIDE_PANEL_WIDTH,
                screen_dimensions.y / 2.0 - TOP_PANEL_HEIGHT,
            ),
            ctx,
        );

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
                    ROW_LABEL_X_OFFSET + count as f32 * step,
                    start_pos + spacing * ((row - row_range.start) as f32 + 1.0),
                ))
                .show(ctx, |ui| {
                    if self.moving_row && row == self.selected_row.unwrap() {
                        Label::new(
                            RichText::new(info)
                                .color(Color32::from_rgba_premultiplied(160, 160, 160, 255)),
                        )
                        .ui(ui);
                    } else {
                        ui.label(info);
                    }
                });
        }
    }

    fn add_buttons(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        row_range: std::ops::Range<usize>,
        screen_dimensions: Vec2,
        moving_let_go_position: Option<Pos2>,
    ) {
        let mut start_pos = 0.0;
        for row in row_range.clone().into_iter() {
            let button_color = if let Some(selected_row) = self.selected_row {
                if selected_row == row {
                    if self.moving_row {
                        Color32::from_rgba_premultiplied(220, 239, 250, 255)
                    } else {
                        Color32::from_rgba_premultiplied(189, 231, 255, 255)
                    }
                } else {
                    Color32::from_rgba_premultiplied(0, 0, 0, if row % 2 == 0 { 10 } else { 30 })
                }
            } else {
                Color32::from_rgba_premultiplied(0, 0, 0, if row % 2 == 0 { 10 } else { 30 })
            };

            let button = Button::new(" ".repeat(1000)).wrap(false).fill(button_color);

            let response = button.ui(ui);

            if let Some(scroll_to_me_row) = self.scroll_to_me_row {
                if scroll_to_me_row == row {
                    response.scroll_to_me(Some(Align::Center));
                    self.scroll_to_me_row = None;
                }
            }

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
                self.modal = self.action_list[self.selected_row.unwrap()]
                    .get_modify_command_window(false, response.hover_pos().unwrap(), ctx);
            }

            if row == row_range.clone().start {
                start_pos = response.rect.top() - response.rect.height();
            }

            if let (Some(moving_let_go_position), Some(selected_row)) =
                (moving_let_go_position, self.selected_row)
            {
                let distance_from_center = response.rect.center().y - moving_let_go_position.y;
                if distance_from_center.abs() < response.rect.height() / 1.7 && selected_row != row
                {
                    if distance_from_center > 0.0 {
                        self.action_list
                            .insert(row, self.action_list[selected_row].clone());

                        if row > selected_row {
                            self.action_list.remove(selected_row);
                            self.selected_row = Some(row - 1);
                        } else {
                            self.action_list.remove(selected_row + 1);
                            self.selected_row = Some(row);
                        }
                    } else {
                        self.action_list
                            .insert(row + 1, self.action_list[selected_row].clone());

                        if row > selected_row {
                            self.action_list.remove(selected_row);
                            self.selected_row = Some(row);
                        } else {
                            self.action_list.remove(selected_row + 1);
                            self.selected_row = Some(row + 1);
                        }
                    }
                }
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
        ctx: &Context,
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

            self.hotkey_detector_sender
                .take()
                .unwrap()
                .send(())
                .unwrap();
            self.hotkey_detector_sender =
                Some(start_hotkey_detector(self.settings.hotkeys.clone()));
        }

        if let (true, Some(..)) = (ui.input().key_pressed(Key::Enter), self.selected_row) {
            self.modal = self.action_list[self.selected_row.unwrap()].get_modify_command_window(
                false,
                pos2(
                    screen_dimensions.x / 2.0 - SIDE_PANEL_WIDTH,
                    screen_dimensions.y / 2.0 - TOP_PANEL_HEIGHT,
                ),
                ctx,
            );
            self.right_click_dialog = None;
        }

        if ui.input().key_pressed(Key::ArrowUp) {
            if let Some(selected_row) = self.selected_row {
                if selected_row > 0 {
                    self.selected_row = Some(selected_row - 1);
                    self.scroll_to_me_row = self.selected_row;
                }
            }
            self.right_click_dialog = None;
        }

        if ui.input().key_pressed(Key::ArrowDown) {
            if let Some(selected_row) = self.selected_row {
                if selected_row < self.action_list.len() - 1 {
                    self.selected_row = Some(selected_row + 1);
                    self.scroll_to_me_row = self.selected_row;
                }
            }
            self.right_click_dialog = None;
        }
    }

    fn are_any_modals_open(&self) -> bool {
        self.modal.is_some()
    }

    fn try_save(&mut self, path: PathBuf, frame: &mut eframe::Frame) {
        let save_result = save_macro(&path, &self.action_list);

        if let Err(error) = save_result {
            self.modal = Some(DefaultErrorWindow::new(
                "Save Error".into(),
                vec!["Error saving macro:".into(), error.to_string()],
            ))
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

    fn handle_moving_row(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        row_height: f32,
        screen_dimensions: Vec2,
    ) {
        let pos = ui.input().pointer.hover_pos().unwrap();

        Area::new("Moving Row Area")
            .order(Order::Background)
            .current_pos(pos2(pos.x - 2000.0, pos.y - row_height / 2.0))
            .enabled(false)
            .show(ctx, |ui| {
                Button::new(" ".repeat(1000))
                    .wrap(false)
                    .fill(Color32::from_rgba_premultiplied(189, 231, 255, 255))
                    .ui(ui);
            });

        let info = self
            .action_list
            .get(self.selected_row.unwrap())
            .unwrap()
            .get_grid_formatted();

        for (count, info) in info.iter().enumerate() {
            let step = (screen_dimensions.x - SIDE_PANEL_WIDTH) / 3.0;

            Area::new(format!("Draggable Row Text {}", count))
                .interactable(false)
                .order(Order::Background)
                .fixed_pos(pos2(
                    ROW_LABEL_X_OFFSET + count as f32 * step,
                    pos.y - row_height / 2.0,
                ))
                .show(ctx, |ui| {
                    Label::new(info).wrap(false).ui(ui);
                });
        }
    }
}
