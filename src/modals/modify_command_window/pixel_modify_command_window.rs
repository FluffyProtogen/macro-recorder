use std::cell::RefCell;

use crate::{
    actions::{Action, PixelInfo},
    gui::{Recorder, PIXELS_PER_POINT},
    images::get_color_under_mouse,
    modals::ModalWindow,
};
use eframe::egui::*;
use winapi::um::winuser::{GetAsyncKeyState, VK_F2};

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
    previous_window_info: Option<(Pos2, Vec2)>,
    screenshot_next_frame: bool,
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
                selected_color: Color32::from_rgba_premultiplied(255, 255, 255, 255),
                rgb_text_edit_texts: ("255".into(), "255".into(), "255".into()),
                f2_previously_pressed: false,

                // FIX THE DEFAULTS TO USE PIXELINFO'S DATA
                capturing: false,
                capture_start: None,
                capture_end: None,
                search_location_text_edit_texts: None,
                move_mouse_if_found: false,
                check_if_not_found: false,
                capturing_screenshot: false,
                previous_window_info: None,
                screenshot_next_frame: false,
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

    /*fn capture(
        data: &mut PixelModifyCommandWindowData,
        frame: &mut eframe::Frame,
        recorder: &mut Recorder,
        capturing_screenshot: bool,
    ) {
        data.capturing = true;
        data.capturing_screenshot = capturing_screenshot;
        data.previous_window_info = Some((
            frame.info().window_info.position.unwrap(),
            frame.info().window_info.size,
        ));
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

        if data.screenshot_next_frame {
            frame.set_decorations(true);
            frame.set_fullscreen(false);
            recorder.transparent = false;

            data.screenshot_next_frame = false;

            let pixels_per_point = PIXELS_PER_POINT.get().unwrap();

            data.capture_start = Some(pos2(
                ((data.capture_start.unwrap().x) * pixels_per_point).round(),
                ((data.capture_start.unwrap().y) * pixels_per_point).round(),
            ));

            data.capture_end = Some(pos2(
                ((data.capture_end.unwrap().x) * pixels_per_point).round(),
                ((data.capture_end.unwrap().y) * pixels_per_point).round(),
            ));

            let capture_start = data.capture_start.unwrap();
            let capture_end = data.capture_end.unwrap();

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

                frame.set_window_size(data.previous_window_info.unwrap().1);
                frame.set_window_pos(data.previous_window_info.unwrap().0);
            }

            frame.set_visible(true);
            data.capturing = false;
            return;
        }

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
                data.screenshot_next_frame = true;
                frame.set_visible(false);
            }
        }
    }*/

    fn save(&self, data: &PixelModifyCommandWindowData, recorder: &mut Recorder) {}

    fn cancel(&self, data: &PixelModifyCommandWindowData, recorder: &mut Recorder) {}
}

impl ModalWindow for PixelModifyCommandWindow {
    fn update(
        &self,
        recorder: &mut Recorder,
        ctx: &Context,
        _ui: &mut Ui,
        drag_bounds: Rect,
        _frame: &mut eframe::Frame,
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
        });
    }
}

fn f2_pressed() -> bool {
    unsafe { GetAsyncKeyState(VK_F2) < 0 }
}
