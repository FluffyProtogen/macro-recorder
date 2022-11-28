use std::cell::RefCell;

use crate::{
    actions::{Action, PixelInfo},
    gui::{Recorder, PIXELS_PER_POINT},
    images::{get_color_under_mouse, screenshot, screenshot_to_color_image},
    modals::ModalWindow,
};
use eframe::egui::*;
use winapi::um::winuser::{
    GetAsyncKeyState, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, VK_F2,
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum CapturingState {
    CaptureNextFrame,
    Capturing,
    NotCapturing,
}

const CAPTURE_COLOR: Color32 = Color32::from_rgba_premultiplied(50, 50, 50, 0);
const INVALID_CAPTURE_COLOR: Color32 = Color32::from_rgba_premultiplied(50, 00, 00, 0);
pub struct PixelModifyCommandWindow {
    data: RefCell<PixelModifyCommandWindowData>,
}
struct PixelModifyCommandWindowData {
    window_type: PixelWindowType,
    creating_command: bool,
    position: Option<Pos2>,
    enter_lock: bool,
    selected_color: Color32,
    rgb_text_edit_texts: (String, String, String),
    f2_previously_pressed: bool,
    capturing: bool,
    capture_start: Option<Pos2>,
    capture_end: Option<Pos2>,
    search_location_text_edit_texts: Option<((String, String), (String, String))>,
    move_mouse_if_found: bool,
    check_if_not_found: bool,
    capturing_screenshot: bool,
    screenshot_next_frame: bool,
    capture_state: CapturingState,
    full_screen_texture: Option<TextureHandle>,
}

#[derive(Clone, Copy)]
pub enum PixelWindowType {
    Wait,
    If,
}

impl PixelModifyCommandWindow {
    pub fn new(
        info: &PixelInfo,
        creating_command: bool,
        position: Pos2,
        window_type: PixelWindowType,
    ) -> Self {
        Self {
            data: RefCell::new(PixelModifyCommandWindowData {
                window_type,
                creating_command,
                position: Some(position),
                enter_lock: true,
                selected_color: Color32::from_rgba_premultiplied(
                    info.color.0,
                    info.color.1,
                    info.color.2,
                    255,
                ),
                rgb_text_edit_texts: (
                    info.color.0.to_string(),
                    info.color.1.to_string(),
                    info.color.2.to_string(),
                ),
                f2_previously_pressed: false,
                capturing: false,
                capture_start: None,
                capture_end: None,
                search_location_text_edit_texts: if creating_command {
                    None
                } else {
                    Some((
                        (
                            info.search_location_left_top.0.to_string(),
                            info.search_location_left_top.1.to_string(),
                        ),
                        (
                            info.search_location_width_height.0.to_string(),
                            info.search_location_width_height.1.to_string(),
                        ),
                    ))
                },
                move_mouse_if_found: info.move_mouse_if_found,
                check_if_not_found: info.check_if_not_found,
                capturing_screenshot: false,
                screenshot_next_frame: false,
                capture_state: CapturingState::NotCapturing,
                full_screen_texture: None,
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        let mut data = self.data.borrow_mut();

        let mut window = Window::new(match data.window_type {
            PixelWindowType::If => "If Pixel",
            PixelWindowType::Wait => "Wait For Pixel",
        })
        .collapsible(false)
        .resizable(false)
        .drag_bounds(drag_bounds);

        if let Some(position) = &data.position {
            window = window.current_pos(Pos2::new(position.x, position.y));
            data.position = None;
        }

        window
    }

    fn capture_begin(
        data: &mut PixelModifyCommandWindowData,
        frame: &mut eframe::Frame,
        recorder: &mut Recorder,
        capturing_screenshot: bool,
    ) {
        data.capturing_screenshot = capturing_screenshot;

        frame.set_decorations(false);
        recorder.transparent = true;
        data.capture_start = None;
        data.capture_end = None;
        data.capture_state = CapturingState::CaptureNextFrame;
        frame.set_visible(false);
    }

    fn not_capturing(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        drag_bounds: Rect,
        frame: &mut eframe::Frame,
    ) {
        let window = self.setup(drag_bounds);

        window.show(ctx, |ui| {
            let data = &mut self.data.borrow_mut();

            if ui.input().key_down(Key::Enter) {
                if !data.enter_lock {
                    self.save(data, recorder);
                }
            } else {
                data.enter_lock = false;
            }

            if ui.input().key_down(Key::Escape) {
                self.cancel(data, recorder);
            }

            if f2_pressed() {
                if !data.f2_previously_pressed {
                    data.selected_color = get_color_under_mouse();
                    data.rgb_text_edit_texts = (
                        data.selected_color.r().to_string(),
                        data.selected_color.g().to_string(),
                        data.selected_color.b().to_string(),
                    );
                    data.f2_previously_pressed = true;
                }
            } else {
                data.f2_previously_pressed = false;
            }

            if let (Ok(r), Ok(g), Ok(b)) = (
                data.rgb_text_edit_texts.0.parse(),
                data.rgb_text_edit_texts.1.parse(),
                data.rgb_text_edit_texts.2.parse(),
            ) {
                data.selected_color = Color32::from_rgba_premultiplied(r, g, b, 255);
            }

            ui.add_space(15.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(20.0);
                ui.label("Selected Color:");
                ui.add_space(15.0);
                color_picker::show_color(ui, data.selected_color, vec2(35.0, 35.0));
            });

            ui.add_space(25.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(10.0);
                ui.label("R: ");
                TextEdit::singleline(&mut data.rgb_text_edit_texts.0)
                    .desired_width(35.0)
                    .ui(ui);
                ui.add_space(10.0);
                ui.label("G: ");
                TextEdit::singleline(&mut data.rgb_text_edit_texts.1)
                    .desired_width(35.0)
                    .ui(ui);
                ui.add_space(10.0);
                ui.label("B: ");
                TextEdit::singleline(&mut data.rgb_text_edit_texts.2)
                    .desired_width(35.0)
                    .ui(ui);
            });

            ui.add_space(25.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(20.0);
                ui.label("Color Under Mouse:");
                ui.add_space(15.0);
                color_picker::show_color(ui, get_color_under_mouse(), vec2(35.0, 35.0));
            });

            ui.add_space(25.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(15.0);
                ui.label("Press F2 to select the color under the mouse");
            });

            ui.add_space(15.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.allocate_space(vec2(45.0, 0.0));

                ui.checkbox(&mut data.move_mouse_if_found, "");
                ui.label("Move mouse to pixel if found");
            });

            ui.allocate_space(vec2(0.0, 15.0));

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.allocate_space(vec2(45.0, 0.0));

                ui.checkbox(&mut data.check_if_not_found, "");
                ui.label("Check if pixel is not found");
            });

            if data.search_location_text_edit_texts.is_none() {
                ui.add_space(15.0);
                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(25.0);
                    if ui.button("Select Area").clicked() {
                        Self::capture_begin(data, frame, recorder, false);
                    }
                });
            }

            if data.search_location_text_edit_texts.is_some() {
                ui.add_space(15.0);

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    let location = &mut data.search_location_text_edit_texts.as_mut().unwrap();
                    ui.allocate_space(vec2(25.0, 0.0));

                    ui.label("Left: ");
                    TextEdit::singleline(&mut location.0 .0)
                        .desired_width(50.0)
                        .ui(ui);
                    ui.allocate_space(vec2(5.0, 0.1));

                    ui.label("Top: ");
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

                    ui.add_space(5.0);

                    if ui.button("Select Area").clicked() {
                        Self::capture_begin(data, frame, recorder, false);
                    }
                });
            }

            ui.add_space(15.0);

            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                ui.add_space(35.0);
                if ui.button("Cancel").clicked() {
                    self.cancel(data, recorder);
                }

                if data.search_location_text_edit_texts.is_some() {
                    ui.add_space(35.0);
                    if ui.button("Save").clicked() {
                        self.save(data, recorder);
                    }
                }
            });
        });
    }

    fn screenshot_this_frame(&self, ctx: &Context, frame: &mut eframe::Frame) {
        let mut data = self.data.borrow_mut();
        let (corner1, corner2) = unsafe {
            (
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            )
        };
        let (corner1, corner2) = (pos2(0.0, 0.0), pos2(corner1 as f32, corner2 as f32));

        let screenshot = screenshot(corner1, corner2);

        data.full_screen_texture = Some(ctx.load_texture(
            "Screenshot Fullscreen",
            screenshot_to_color_image(screenshot.clone()),
            TextureFilter::Linear,
        ));

        data.capture_state = CapturingState::Capturing;

        frame.set_visible(true);
        frame.set_fullscreen(true);
    }

    fn draw_capturing_window(
        &self,
        _ui: &mut Ui,
        frame: &mut eframe::Frame,
        recorder: &mut Recorder,
        ctx: &Context,
    ) {
        let mut data = self.data.borrow_mut();
        let texture = data.full_screen_texture.as_ref().unwrap();

        Area::new("Whole Screenshot")
            .interactable(false)
            .order(Order::Background)
            .fixed_pos(pos2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.image(texture, frame.info().window_info.size);
            });

        Area::new("Overlay")
            .interactable(false)
            .order(Order::Foreground)
            .fixed_pos(pos2(0.0, 0.0))
            .show(ctx, |ui| {
                let screen_size = frame.info().window_info.size;
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
                    if let (Some(current_pos), true) = (pointer.hover_pos(), pointer.primary_down())
                    {
                        data.capture_end = Some(current_pos);

                        let capture_color = CAPTURE_COLOR;

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
                        let pixels_per_point = PIXELS_PER_POINT.get().unwrap();
                        let capture_start = data.capture_start.unwrap();
                        let capture_start = pos2(
                            capture_start.x * pixels_per_point,
                            capture_start.y * pixels_per_point,
                        );

                        let capture_end = data.capture_end.unwrap();
                        let capture_end = pos2(
                            capture_end.x * pixels_per_point,
                            capture_end.y * pixels_per_point,
                        );

                        if (capture_start.x - capture_end.x).abs() as i32 != 0
                            && (capture_start.y - capture_end.y).abs() as i32 != 0
                        {
                            data.search_location_text_edit_texts = Some((
                                (
                                    lesser(capture_start.x, capture_end.x).to_string(),
                                    lesser(capture_start.y, capture_end.y).to_string(),
                                ),
                                (
                                    (capture_start.x - capture_end.x).abs().to_string(),
                                    (capture_start.y - capture_end.y).abs().to_string(),
                                ),
                            ));
                        }

                        data.full_screen_texture = None;
                        data.capture_state = CapturingState::NotCapturing;
                        frame.set_fullscreen(false);
                        frame.set_decorations(true);
                        recorder.transparent = false;
                    }
                }
            });
    }

    fn save(&self, data: &PixelModifyCommandWindowData, recorder: &mut Recorder) {
        if let Some(texts) = &data.search_location_text_edit_texts {
            if let (Ok(left), Ok(top), Ok(width), Ok(height)) = (
                texts.0 .0.parse(),
                texts.0 .1.parse(),
                texts.1 .0.parse(),
                texts.1 .1.parse(),
            ) {
                recorder.modal = None;

                let pixel_info = PixelInfo {
                    color: (
                        data.selected_color.r(),
                        data.selected_color.g(),
                        data.selected_color.b(),
                    ),
                    search_location_left_top: (left, top),
                    search_location_width_height: (width, height),
                    check_if_not_found: data.check_if_not_found,
                    move_mouse_if_found: data.move_mouse_if_found,
                };

                let selected_row = recorder.selected_row.unwrap();
                recorder.action_list()[selected_row] = match data.window_type {
                    PixelWindowType::If => Action::IfPixel(pixel_info),
                    PixelWindowType::Wait => Action::WaitForPixel(pixel_info),
                };
            }
        }
    }

    fn cancel(&self, data: &PixelModifyCommandWindowData, recorder: &mut Recorder) {
        let selected_row = recorder.selected_row.unwrap();
        recorder.modal = None;
        if data.creating_command {
            recorder.action_list().remove(selected_row);
            recorder.selected_row = None;
        }
    }
}

impl ModalWindow for PixelModifyCommandWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        ui: &mut Ui,
        drag_bounds: Rect,
        frame: &mut eframe::Frame,
    ) {
        let state = self.data.borrow().capture_state;

        match state {
            CapturingState::CaptureNextFrame => self.screenshot_this_frame(ctx, frame),
            CapturingState::Capturing => self.draw_capturing_window(ui, frame, recorder, ctx),
            CapturingState::NotCapturing => self.not_capturing(recorder, ctx, drag_bounds, frame),
        }
    }
}

fn f2_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_F2) < 0 }
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
