use crate::{
    actions::Action,
    gui::Recorder,
    images::{screenshot, screenshot_to_color_image, RawScreenshot, IMAGE_PANEL_IMAGE_SIZE},
};
use std::cell::RefCell;

use super::ModifyCommandWindow;
use crate::gui::PIXELS_PER_POINT;
use eframe::egui::*;
use winapi::{
    shared::winerror::ERROR_SINGLE_INSTANCE_APP,
    um::{
        propidl::PID_MAX_READONLY,
        winuser::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN},
    },
};

const CAPTURE_COLOR: Color32 = Color32::from_rgba_premultiplied(50, 50, 50, 0);
const INVALID_CAPTURE_COLOR: Color32 = Color32::from_rgba_premultiplied(50, 00, 00, 0);

pub struct WaitForImageModifyCommandWindow {
    data: RefCell<WaitForImageModifyCommandWindowData>,
}

struct WaitForImageModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    capturing: bool,
    capture_start: Option<Pos2>,
    capture_end: Option<Pos2>,
    screenshot_raw: Option<RawScreenshot>,
    screenshot_texture: Option<TextureHandle>,
    max_difference_text_edit_text: String,
    search_location_text_edit_texts: Option<((String, String), (String, String))>,
    move_mouse_if_found: bool,
    check_if_not_found: bool,
    capturing_screenshot: bool,
}

impl WaitForImageModifyCommandWindow {
    pub fn new(creating_command: bool, position: Pos2) -> Self {
        Self {
            data: RefCell::new(WaitForImageModifyCommandWindowData {
                creating_command,
                position: Some(position),
                capturing: false,
                capture_start: None,
                capture_end: None,
                screenshot_raw: None,
                screenshot_texture: None,
                max_difference_text_edit_text: "0".into(),
                search_location_text_edit_texts: None,
                move_mouse_if_found: false,
                check_if_not_found: false,
                capturing_screenshot: false,
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        let mut window = Window::new("Wait For Image")
            .collapsible(false)
            .resizable(false)
            .drag_bounds(drag_bounds);

        let mut data = self.data.borrow_mut();

        if let Some(position) = &data.position {
            window = window.current_pos(Pos2::new(position.x, position.y));
            data.position = None;
        }

        window
    }

    fn save(&self, data: &WaitForImageModifyCommandWindowData, recorder: &mut Recorder) {}

    fn cancel(&self, data: &WaitForImageModifyCommandWindowData, recorder: &mut Recorder) {}

    fn capture(
        data: &mut WaitForImageModifyCommandWindowData,
        frame: &mut eframe::Frame,
        recorder: &mut Recorder,
        capturing_screenshot: bool,
    ) {
        data.capturing = true;
        data.capturing_screenshot = capturing_screenshot;
        frame.set_decorations(false);
        frame.set_window_pos(pos2(-1.0, 0.0)); // offset by 1 so windows doesn't stop rendering everything behind the window lol

        let (width, height) = unsafe {
            (
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            )
        };

        let pixels_per_point = PIXELS_PER_POINT.get().unwrap();

        frame.set_window_size(vec2(
            (width as f32 / pixels_per_point) + 1.0,
            height as f32 / pixels_per_point,
        )); // add 1 to it to compensate

        recorder.transparent = true;
        data.capture_start = None;
        data.capture_end = None;
    }

    fn draw_capturing_window(
        &self,
        ui: &mut Ui,
        frame: &mut eframe::Frame,
        recorder: &mut Recorder,
        ctx: &Context,
    ) {
        //make x -1 but make window 1 pixel longer and offest the mouse capture coordinates by 1

        let screen_size = frame.info().window_info.size;

        let data = &mut self.data.borrow_mut();

        let pointer = ui.input().pointer.clone(); // clone it to avoid a dead lock

        if let (true, Some(pos)) = (pointer.primary_clicked(), pointer.hover_pos()) {
            data.capture_start = Some(pos);
        } else if data.capture_start.is_none() {
            ui.painter().rect_filled(
                Rect::from_two_pos(
                    Pos2::ZERO,
                    Pos2 {
                        x: frame.info().window_info.size.x,
                        y: frame.info().window_info.size.x,
                    },
                ),
                Rounding::none(),
                CAPTURE_COLOR,
            );
        }

        if let Some(initial_pos) = data.capture_start {
            if let (Some(current_pos), true) = (pointer.hover_pos(), pointer.primary_down()) {
                data.capture_end = Some(current_pos);

                let capture_color = if data.capturing_screenshot {
                    CAPTURE_COLOR
                } else {
                    let screenshot_size = data.screenshot_texture.as_ref().unwrap().size_vec2();
                    let pixels_per_point = PIXELS_PER_POINT.get().unwrap();
                    if (initial_pos.x - current_pos.x).abs() * pixels_per_point > screenshot_size.x
                        && (initial_pos.y - current_pos.y).abs() * pixels_per_point
                            > screenshot_size.y
                    {
                        CAPTURE_COLOR
                    } else {
                        INVALID_CAPTURE_COLOR
                    }
                };

                if initial_pos.y == current_pos.y {
                    ui.painter().rect_filled(
                        Rect::from_two_pos(
                            Pos2::ZERO,
                            Pos2 {
                                x: frame.info().window_info.size.x,
                                y: frame.info().window_info.size.x,
                            },
                        ),
                        Rounding::none(),
                        capture_color,
                    );
                } else {
                    ui.painter().rect_filled(
                        Rect::from_x_y_ranges(
                            0.0..=lesser(initial_pos.x, current_pos.x),
                            lesser(initial_pos.y, current_pos.y)
                                ..=greater(initial_pos.y, current_pos.y),
                        ),
                        Rounding::none(),
                        capture_color,
                    );

                    ui.painter().rect_filled(
                        Rect::from_x_y_ranges(
                            greater(initial_pos.x, current_pos.x)..=screen_size.x,
                            lesser(initial_pos.y, current_pos.y)
                                ..=greater(initial_pos.y, current_pos.y),
                        ),
                        Rounding::none(),
                        capture_color,
                    );

                    ui.painter().rect_filled(
                        Rect::from_x_y_ranges(
                            0.0..=screen_size.x,
                            0.0..=lesser(initial_pos.y, current_pos.y),
                        ),
                        Rounding::none(),
                        capture_color,
                    );

                    ui.painter().rect_filled(
                        Rect::from_x_y_ranges(
                            0.0..=screen_size.x,
                            greater(initial_pos.y, current_pos.y)..=screen_size.y,
                        ),
                        Rounding::none(),
                        capture_color,
                    );
                }
            } else {
                data.capturing = false;
                frame.set_decorations(true);
                frame.set_fullscreen(false);
                recorder.transparent = false;

                let pixels_per_point = PIXELS_PER_POINT.get().unwrap();

                data.capture_start = Some(pos2(
                    ((data.capture_start.unwrap().x - 1.0) * pixels_per_point).round(),
                    ((data.capture_start.unwrap().y - 1.0) * pixels_per_point).round(),
                ));

                data.capture_end = Some(pos2(
                    ((data.capture_end.unwrap().x - 1.0) * pixels_per_point).round(),
                    ((data.capture_end.unwrap().y - 1.0) * pixels_per_point).round(),
                ));

                let capture_start = data.capture_start.unwrap();
                let capture_end = data.capture_end.unwrap();

                if (capture_start.x - capture_end.x).abs() as i32 != 0
                    && (capture_start.y - capture_end.y).abs() as i32 != 0
                {
                    if data.capturing_screenshot {
                        let screenshot = screenshot(capture_start, capture_end);
                        data.screenshot_texture = Some(ctx.load_texture(
                            "Screenshot",
                            screenshot_to_color_image(screenshot.clone()),
                            TextureFilter::Linear,
                        ));
                        data.screenshot_raw = Some(screenshot);
                    }

                    data.search_location_text_edit_texts = Some((
                        (
                            lesser(capture_start.x, capture_end.x).to_string(),
                            greater(capture_start.x, capture_end.x).to_string(),
                        ),
                        (
                            (capture_start.x - capture_end.x).abs().to_string(),
                            (capture_start.y - capture_end.y).abs().to_string(),
                        ),
                    ));
                }
            }
        }
    }
}

impl ModifyCommandWindow for WaitForImageModifyCommandWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        drag_bounds: Rect,
        frame: &mut eframe::Frame,
    ) {
        if self.data.borrow().capturing {
            self.draw_capturing_window(ui, frame, recorder, ctx);
        } else {
            let window = self.setup(drag_bounds);
            let data = &mut self.data.borrow_mut();

            window.show(ctx, |ui| {
                ui.allocate_space(vec2(0.0, 15.0));

                if let Some(texture) = &data.screenshot_texture {
                    ui.painter().rect_filled(
                        Rect::from_x_y_ranges(
                            *ui.cursor().x_range().start()
                                ..=(ui.cursor().x_range().start() + IMAGE_PANEL_IMAGE_SIZE),
                            *ui.cursor().y_range().start()
                                ..=(ui.cursor().y_range().start() + IMAGE_PANEL_IMAGE_SIZE),
                        ),
                        Rounding::none(),
                        Color32::from_rgba_premultiplied(217, 217, 217, 255),
                    );

                    let texture_size = texture.size_vec2();

                    let (width, height) = if texture_size.x > texture_size.y {
                        let scale_factor = IMAGE_PANEL_IMAGE_SIZE / texture_size.x;
                        (texture_size.x * scale_factor, texture_size.y * scale_factor)
                    } else {
                        let scale_factor = IMAGE_PANEL_IMAGE_SIZE / texture_size.y;
                        (texture_size.x * scale_factor, texture_size.y * scale_factor)
                    };

                    ui.allocate_space(vec2(0.0, (IMAGE_PANEL_IMAGE_SIZE - height) / 2.0));
                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.allocate_space(vec2((IMAGE_PANEL_IMAGE_SIZE - width) / 2.0, 0.0));
                        ui.image(texture, vec2(width, height));
                        ui.allocate_space(vec2((IMAGE_PANEL_IMAGE_SIZE - width) / 2.0, 0.0));
                    });
                    ui.allocate_space(vec2(0.0, (IMAGE_PANEL_IMAGE_SIZE - height) / 2.0));
                }

                ui.allocate_space(vec2(0.0, 15.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.allocate_space(vec2(45.0, 0.0));

                    if ui.button("Select Image").clicked() {
                        Self::capture(data, frame, recorder, true);
                    }
                });

                if data.screenshot_texture.is_some() {
                    ui.allocate_space(vec2(0.0, 15.0));

                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.allocate_space(vec2(45.0, 0.0));

                        TextEdit::singleline(&mut data.max_difference_text_edit_text)
                            .desired_width(50.0)
                            .ui(ui);
                        ui.allocate_space(vec2(5.0, 0.0));
                        ui.label("Max Difference (0 means identical)");
                    });

                    ui.allocate_space(vec2(0.0, 15.0));

                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.allocate_space(vec2(45.0, 0.0));

                        ui.checkbox(&mut data.move_mouse_if_found, "");
                        ui.label("Move mouse to center of image if found");
                    });

                    ui.allocate_space(vec2(0.0, 15.0));

                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.allocate_space(vec2(45.0, 0.0));

                        ui.checkbox(&mut data.check_if_not_found, "");
                        ui.label("Check if image is not found");
                    });

                    ui.allocate_space(vec2(0.0, 15.0));

                    ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                        ui.allocate_space(vec2(45.0, 0.0));

                        if ui
                            .add(Checkbox::new(
                                &mut data.search_location_text_edit_texts.is_none(),
                                "",
                            ))
                            .clicked()
                        {
                            if data.search_location_text_edit_texts.is_none() {
                                data.search_location_text_edit_texts = Some((
                                    (String::new(), String::new()),
                                    (String::new(), String::new()),
                                ));
                            } else {
                                data.search_location_text_edit_texts = None;
                            }
                        }
                        ui.label("Search the whole screen for the image");
                    });

                    ui.allocate_space(vec2(0.0, 15.0));

                    if data.search_location_text_edit_texts.is_some() {
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            let location =
                                &mut data.search_location_text_edit_texts.as_mut().unwrap();
                            ui.allocate_space(vec2(45.0, 0.0));

                            ui.label("Top: ");
                            TextEdit::singleline(&mut location.0 .0)
                                .desired_width(50.0)
                                .ui(ui);
                            ui.allocate_space(vec2(5.0, 0.1));

                            ui.label("Left: ");
                            TextEdit::singleline(&mut location.0 .1)
                                .desired_width(50.0)
                                .ui(ui);
                            ui.allocate_space(vec2(5.0, 0.0));

                            ui.label("Width: ");
                            TextEdit::singleline(&mut location.1 .0)
                                .desired_width(50.0)
                                .ui(ui);
                            ui.allocate_space(vec2(5.0, 0.0));

                            ui.label("Height: ");
                            TextEdit::singleline(&mut location.1 .1)
                                .desired_width(50.0)
                                .ui(ui);
                            ui.allocate_space(vec2(15.0, 0.0));

                            if ui.button("Select Area").clicked() {
                                Self::capture(data, frame, recorder, false);
                            }
                        });

                        ui.allocate_space(vec2(0.0, 15.0));

                        if ui.button("Check if image is found").clicked() {}
                    }

                    if ui.input().key_pressed(Key::Enter) {
                        self.save(data, recorder);
                    }
                    if ui.input().key_pressed(Key::Escape) {
                        self.cancel(data, recorder);
                    }
                }
            });
        }
    }
}

fn greater(one: f32, two: f32) -> f32 {
    if one > two {
        one
    } else {
        two
    }
}

fn lesser(one: f32, two: f32) -> f32 {
    if one < two {
        one
    } else {
        two
    }
}
