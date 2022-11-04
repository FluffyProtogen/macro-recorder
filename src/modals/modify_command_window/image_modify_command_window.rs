use crate::gui::PIXELS_PER_POINT;
use crate::images::RawScreenshot;
use crate::modals::ModalWindow;
use crate::{
    actions::{Action, ImageInfo},
    gui::Recorder,
    images::{
        find_image, screenshot, screenshot_to_color_image, GrayImageSerializable,
        RawScreenshotPair, IMAGE_PANEL_IMAGE_SIZE,
    },
};
use eframe::egui::*;
use image::{DynamicImage, EncodableLayout, ImageBuffer};
use std::cell::RefCell;
use winapi::um::winuser::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};

const CAPTURE_COLOR: Color32 = Color32::from_rgba_premultiplied(50, 50, 50, 0);
const INVALID_CAPTURE_COLOR: Color32 = Color32::from_rgba_premultiplied(50, 00, 00, 0);

#[derive(Clone, Copy)]
pub enum ImageWindowType {
    Wait,
    If,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum CapturingState {
    CaptureNextFrame,
    Capturing,
    NotCapturing,
}

pub struct ImageModifyCommandWindow {
    data: RefCell<ImageModifyCommandWindowData>,
}

struct ImageModifyCommandWindowData {
    creating_command: bool,
    position: Option<Pos2>,
    capture_start: Option<Pos2>,
    capture_end: Option<Pos2>,
    screenshot_raw: Option<RawScreenshotPair>,
    screenshot_texture: Option<TextureHandle>,
    max_difference_text_edit_text: String,
    search_location_text_edit_texts: Option<((String, String), (String, String))>,
    move_mouse_if_found: bool,
    check_if_not_found: bool,
    capturing_screenshot: bool,
    enter_lock: bool,
    window_type: ImageWindowType,
    full_screen_texture: Option<TextureHandle>,
    full_screen_image: Option<ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    capture_state: CapturingState,
}

impl ImageModifyCommandWindow {
    pub fn new(
        image_info: &ImageInfo,
        creating_command: bool,
        position: Pos2,
        ctx: &Context,
        window_type: ImageWindowType,
    ) -> Self {
        let search_location_text_edit_texts = match (
            image_info.search_location_left_top,
            image_info.search_location_width_height,
        ) {
            (Some(top_left), Some(width_height)) => Some((
                (top_left.0.to_string(), top_left.1.to_string()),
                (width_height.0.to_string(), width_height.1.to_string()),
            )),
            _ => None,
        };

        Self {
            data: RefCell::new(ImageModifyCommandWindowData {
                creating_command,
                position: Some(position),
                capture_start: None,
                capture_end: None,
                screenshot_raw: image_info.screenshot_raw.clone(),
                screenshot_texture: image_info.screenshot_raw.as_ref().map(|screenshot| {
                    ctx.load_texture(
                        "Screenshot",
                        screenshot_to_color_image(screenshot.color.clone()),
                        TextureFilter::Linear,
                    )
                }),
                max_difference_text_edit_text: image_info.image_similarity.to_string(),
                search_location_text_edit_texts,
                move_mouse_if_found: image_info.move_mouse_if_found,
                check_if_not_found: image_info.check_if_not_found,
                capturing_screenshot: false,
                enter_lock: true,
                window_type,
                full_screen_texture: None,
                capture_state: CapturingState::NotCapturing,
                full_screen_image: None,
            }),
        }
    }

    fn setup(&self, drag_bounds: Rect) -> Window {
        let mut data = self.data.borrow_mut();

        let mut window = Window::new(match data.window_type {
            ImageWindowType::Wait => "Wait For Image",
            ImageWindowType::If => "If Image Found",
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

    fn save(&self, data: &ImageModifyCommandWindowData, recorder: &mut Recorder) {
        let location_size = match &data.search_location_text_edit_texts {
            Some(texts) => {
                if let (Ok(left), Ok(top), Ok(width), Ok(height)) = (
                    texts.0 .0.parse(),
                    texts.0 .1.parse(),
                    texts.1 .0.parse(),
                    texts.1 .1.parse(),
                ) {
                    Some(((left, top), (width, height)))
                } else {
                    return;
                }
            }
            None => None,
        };

        let search_location_left_top = location_size.map(|location_size| location_size.0);
        let search_location_width_height = location_size.map(|location_size| location_size.1);

        let image_similarity = match data.max_difference_text_edit_text.parse() {
            Ok(result) => result,
            Err(..) => return,
        };

        if let Some(screenshot_raw) = data.screenshot_raw.clone() {
            let image_info = ImageInfo {
                screenshot_raw: Some(screenshot_raw),
                move_mouse_if_found: data.move_mouse_if_found,
                check_if_not_found: data.check_if_not_found,
                search_location_left_top,
                search_location_width_height,
                image_similarity,
            };

            recorder.modal = None;
            recorder.action_list[recorder.selected_row.unwrap()] = match data.window_type {
                ImageWindowType::Wait => Action::WaitForImage(image_info),
                ImageWindowType::If => Action::IfImage(image_info),
            };
        }
    }

    fn cancel(&self, data: &ImageModifyCommandWindowData, recorder: &mut Recorder) {
        recorder.modal = None;
        if data.creating_command {
            recorder.action_list.remove(recorder.selected_row.unwrap());
            recorder.selected_row = None;
        }
    }

    fn capture_begin(
        data: &mut ImageModifyCommandWindowData,
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

                        let capture_color = if data.capturing_screenshot {
                            CAPTURE_COLOR
                        } else {
                            let screenshot_size =
                                data.screenshot_texture.as_ref().unwrap().size_vec2();
                            let pixels_per_point = PIXELS_PER_POINT.get().unwrap();
                            if (initial_pos.x - current_pos.x).abs() * pixels_per_point
                                > screenshot_size.x
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
                            if data.capturing_screenshot {
                                let mut image = data.full_screen_image.take().unwrap();
                                let cropped = image::imageops::crop(
                                    &mut image,
                                    lesser(capture_start.x, capture_end.x) as u32,
                                    lesser(capture_start.y, capture_end.y) as u32,
                                    (capture_start.x - capture_end.x).abs() as u32,
                                    (capture_start.y - capture_end.y).abs() as u32,
                                )
                                .to_image();

                                let raw_pair = RawScreenshotPair {
                                    color: RawScreenshot {
                                        pixels: cropped.clone().as_bytes().to_vec(),
                                        width: cropped.width() as usize,
                                        height: cropped.height() as usize,
                                        x: lesser(capture_start.x, capture_end.x) as i32,
                                        y: lesser(capture_start.y, capture_end.y) as i32,
                                    },
                                    gray: GrayImageSerializable(
                                        DynamicImage::ImageRgba8(cropped).to_luma8(),
                                    ),
                                };

                                data.screenshot_raw = Some(raw_pair);

                                data.screenshot_texture = Some(ctx.load_texture(
                                    "Screenshot",
                                    screenshot_to_color_image(
                                        data.screenshot_raw.as_ref().unwrap().color.clone(),
                                    ),
                                    TextureFilter::Linear,
                                ));
                            }
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
}

impl ImageModifyCommandWindow {
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

        let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_vec(
            screenshot.width as u32,
            screenshot.height as u32,
            screenshot.pixels,
        );

        data.full_screen_image = Some(buffer.unwrap());

        data.capture_state = CapturingState::Capturing;

        frame.set_visible(true);
        frame.set_fullscreen(true);
    }

    fn not_capturing(
        &self,
        drag_bounds: Rect,
        recorder: &mut Recorder,
        ctx: &Context,
        frame: &mut eframe::Frame,
        ui: &mut Ui,
    ) {
        let window = self.setup(drag_bounds);
        let data = &mut self.data.borrow_mut();

        if ui.input().key_pressed(Key::Escape) {
            self.cancel(data, recorder);
        }

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
                    Self::capture_begin(data, frame, recorder, true);
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
                    ui.label("Similarity (1 means identical; 0.97 recommended)");
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
                        let location = &mut data.search_location_text_edit_texts.as_mut().unwrap();
                        ui.allocate_space(vec2(45.0, 0.0));

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
                        ui.allocate_space(vec2(15.0, 0.0));

                        if ui.button("Select Area").clicked() {
                            Self::capture_begin(data, frame, recorder, false);
                        }
                    });

                    ui.allocate_space(vec2(0.0, 15.0));
                }

                if ui.button("Check if image is found").clicked() {
                    if let Some(text) = &data.search_location_text_edit_texts {
                        let start = match (text.0 .0.parse(), text.0 .1.parse()) {
                            (Ok(x), Ok(y)) => Some(pos2(x, y)),
                            _ => None,
                        };
                        let width_height = match (text.1 .0.parse(), text.1 .1.parse()) {
                            (Ok(x), Ok(y)) => Some(pos2(x, y)),
                            _ => None,
                        };

                        if let (Some(start), Some(width_height)) = (start, width_height) {
                            let end = pos2(start.x + width_height.x, start.y + width_height.y);
                            find_image(data.screenshot_raw.as_ref().unwrap(), Some((start, end)));
                        }
                    } else {
                        find_image(data.screenshot_raw.as_ref().unwrap(), None);
                    };
                }

                if ui.input().key_down(Key::Enter) {
                    if !data.enter_lock {
                        self.save(data, recorder);
                    }
                } else {
                    data.enter_lock = false;
                }

                ui.allocate_space(vec2(0.0, 15.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(35.0);
                    if ui.button("Cancel").clicked() {
                        self.cancel(data, recorder);
                    }
                    ui.add_space(35.0);
                    if ui.button("Save").clicked() {
                        self.save(data, recorder);
                    }
                });
            } else {
                ui.allocate_space(vec2(0.0, 15.0));

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(35.0);
                    if ui.button("Cancel").clicked() {
                        self.cancel(data, recorder);
                    }
                });
            }
        });
    }
}

impl ModalWindow for ImageModifyCommandWindow {
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
            CapturingState::NotCapturing => {
                self.not_capturing(drag_bounds, recorder, ctx, frame, ui)
            }
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
