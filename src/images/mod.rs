use egui::{pos2, Color32, ColorImage, Pos2};
use image::imageops::{resize, FilterType};
use image::*;
use imageproc::template_matching::{find_extremes, MatchTemplateMethod};
use rayon::prelude::IndexedParallelIterator;
use rayon::slice::ParallelSlice;
use rayon::{prelude::ParallelIterator, slice::ParallelSliceMut};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use std::ffi::c_void;
use std::mem::zeroed;
use std::ptr::null_mut;
use winapi::shared::windef::POINT;
use winapi::um::{wingdi::*, winuser::*};

const RESIZE_FACTOR: u32 = 3;

use serde::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawScreenshot {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawScreenshotPair {
    pub color: RawScreenshot,
    pub gray: GrayImageSerializable,
}

#[derive(Clone, Debug)]
pub struct GrayImageSerializable(pub GrayImage);

impl std::ops::Deref for GrayImageSerializable {
    type Target = GrayImage;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Serialization / Deserialization code copied from the serde examples lol
impl Serialize for GrayImageSerializable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GrayImage", 3)?;
        state.serialize_field("pixels", &self.as_bytes())?;
        state.serialize_field("width", &self.width())?;
        state.serialize_field("height", &self.height())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for GrayImageSerializable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase")]
        enum Field {
            Pixels,
            Width,
            Height,
        }

        struct GrayImageSerializableVisitor;

        impl<'de> Visitor<'de> for GrayImageSerializableVisitor {
            type Value = GrayImageSerializable;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct GrayImageSerializable")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<GrayImageSerializable, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let pixels = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(GrayImageSerializable(
                    ImageBuffer::from_vec(width, height, pixels).unwrap(),
                ))
            }

            fn visit_map<V>(self, mut map: V) -> Result<GrayImageSerializable, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pixels = None;
                let mut width = None;
                let mut height = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Pixels => {
                            if pixels.is_some() {
                                return Err(de::Error::duplicate_field("pixels"));
                            }
                            pixels = Some(map.next_value()?);
                        }
                        Field::Width => {
                            if width.is_some() {
                                return Err(de::Error::duplicate_field("width"));
                            }
                            width = Some(map.next_value()?);
                        }
                        Field::Height => {
                            if height.is_some() {
                                return Err(de::Error::duplicate_field("height"));
                            }
                            height = Some(map.next_value()?);
                        }
                    }
                }
                let pixels = pixels.ok_or_else(|| de::Error::missing_field("pixels"))?;
                let width = width.ok_or_else(|| de::Error::missing_field("width"))?;
                let height = height.ok_or_else(|| de::Error::missing_field("height"))?;
                Ok(GrayImageSerializable(
                    ImageBuffer::from_vec(width, height, pixels).unwrap(),
                ))
            }
        }

        const FIELDS: &'static [&'static str] = &["pixels", "width", "height"];
        deserializer.deserialize_struct(
            "GrayImageSerializable",
            FIELDS,
            GrayImageSerializableVisitor,
        )
    }
}

impl RawScreenshot {
    pub fn to_rgba8(mut self) -> Self {
        self.pixels.par_chunks_mut(4).for_each(|bgra| {
            let blue = bgra[0];
            bgra[0] = bgra[2];
            bgra[2] = blue;
        });
        self
    }
}

impl Clone for RawScreenshot {
    fn clone(&self) -> Self {
        Self {
            pixels: self.pixels.clone(),
            width: self.width,
            height: self.height,
            x: self.x,
            y: self.y,
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

pub const IMAGE_PANEL_IMAGE_SIZE: f32 = 225.0;

// From https://stackoverflow.com/questions/3291167/how-can-i-take-a-screenshot-in-a-windows-application
pub fn screenshot(corner1: Pos2, corner2: Pos2) -> RawScreenshot {
    unsafe {
        let left_x = lesser(corner1.x, corner2.x) as i32;
        let top_y = lesser(corner1.y, corner2.y) as i32;

        let width = (corner1.x - corner2.x).abs() as i32;
        let height = (corner1.y - corner2.y).abs() as i32;

        let dc_screen = GetDC(null_mut());
        let dc_target = CreateCompatibleDC(dc_screen);
        let bmp_target = CreateCompatibleBitmap(dc_screen, width, height);
        let old_bmp = SelectObject(dc_target, bmp_target as *mut c_void);
        BitBlt(
            dc_target,
            0,
            0,
            width,
            height,
            dc_screen,
            left_x,
            top_y,
            SRCCOPY | CAPTUREBLT,
        );
        SelectObject(dc_target, old_bmp);
        DeleteDC(dc_target);
        ReleaseDC(null_mut(), dc_screen);

        let mut pixels = vec![0u8; (width * height) as usize * 4];
        GetBitmapBits(
            bmp_target,
            width * height * 4,
            pixels.as_mut_ptr() as *mut c_void,
        );

        RawScreenshot {
            pixels,
            width: width as usize,
            height: height as usize,
            x: left_x,
            y: top_y,
        }
    }
}

pub fn screenshot_to_color_image(screenshot: RawScreenshot) -> ColorImage {
    let RawScreenshot {
        pixels: mut pixels_bgra,
        width,
        height,
        ..
    } = screenshot;

    pixels_bgra.par_chunks_mut(4).for_each(|bgra| {
        let blue = bgra[0];
        bgra[0] = bgra[2];
        bgra[2] = blue;
    });

    ColorImage::from_rgba_unmultiplied([width, height], &pixels_bgra)
}

// From https://stackoverflow.com/questions/3291167/how-can-i-take-a-screenshot-in-a-windows-application
pub fn screenshot_raw(corner1: Pos2, corner2: Pos2) -> Vec<u8> {
    unsafe {
        let left_x = lesser(corner1.x, corner2.x) as i32;
        let top_y = lesser(corner1.y, corner2.y) as i32;

        let width = (corner1.x - corner2.x).abs() as i32;
        let height = (corner1.y - corner2.y).abs() as i32;

        let dc_screen = GetDC(null_mut());
        let dc_target = CreateCompatibleDC(dc_screen);
        let bmp_target = CreateCompatibleBitmap(dc_screen, width, height);
        let old_bmp = SelectObject(dc_target, bmp_target as *mut c_void);
        BitBlt(
            dc_target,
            0,
            0,
            width,
            height,
            dc_screen,
            left_x,
            top_y,
            SRCCOPY | CAPTUREBLT,
        );
        SelectObject(dc_target, old_bmp);
        DeleteDC(dc_target);
        ReleaseDC(null_mut(), dc_screen);

        let mut pixels = vec![0u8; (width * height) as usize * 4];
        GetBitmapBits(
            bmp_target,
            width * height * 4,
            pixels.as_mut_ptr() as *mut c_void,
        );

        pixels
    }
}

pub fn find_image(
    image: &RawScreenshotPair,
    search_coordinates: Option<(Pos2, Pos2)>,
) -> (f32, (i32, i32)) {
    let search_coordinates = search_coordinates.unwrap_or_else(|| {
        let (width, height) = unsafe {
            (
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            )
        };
        (pos2(0.0, 0.0), pos2(width as f32, height as f32))
    });

    let screenshot = screenshot_raw(search_coordinates.0, search_coordinates.1);

    let screenshot = DynamicImage::ImageRgba8(
        ImageBuffer::from_vec(
            (search_coordinates.0.x - search_coordinates.1.x).abs() as u32,
            (search_coordinates.0.y - search_coordinates.1.y).abs() as u32,
            screenshot,
        )
        .unwrap(),
    )
    .to_luma8();

    let screenshot = resize(
        &screenshot,
        screenshot.width() / RESIZE_FACTOR,
        screenshot.height() / RESIZE_FACTOR,
        FilterType::Gaussian,
    );

    let image = resize(
        &image.gray.0,
        image.gray.width() / RESIZE_FACTOR,
        image.gray.height() / RESIZE_FACTOR,
        FilterType::Gaussian,
    );

    let result = imageproc::template_matching::match_template(
        &screenshot,
        &image,
        MatchTemplateMethod::CrossCorrelationNormalized,
    );

    let result = find_extremes(&result);

    let found_x = lesser(search_coordinates.0.x, search_coordinates.1.x) as i32
        + image.width() as i32 / 2 * RESIZE_FACTOR as i32
        + result.max_value_location.0 as i32 * RESIZE_FACTOR as i32;
    let found_y = lesser(search_coordinates.0.y, search_coordinates.1.y) as i32
        + image.height() as i32 / 2 * RESIZE_FACTOR as i32
        + result.max_value_location.1 as i32 * RESIZE_FACTOR as i32;

    (result.max_value, (found_x, found_y))
}

pub fn find_pixel(search_coordinates: (Pos2, Pos2), color: (u8, u8, u8)) -> Option<(i32, i32)> {
    let width = (search_coordinates.0.x - search_coordinates.1.x).abs() as usize;

    screenshot_raw(search_coordinates.0, search_coordinates.1)
        .par_chunks(4)
        .position_first(|bgra| (bgra[2], bgra[1], bgra[0]) == (color.0, color.1, color.2))
        .map(|index| {
            (
                (index % width) as i32
                    + lesser(search_coordinates.0.x, search_coordinates.1.x) as i32,
                (index / width) as i32
                    + lesser(search_coordinates.0.y, search_coordinates.1.y) as i32,
            )
        })
}

pub fn fast_find_image(
    image: &RawScreenshotPair,
    search_coordinates: Option<(Pos2, Pos2)>,
) -> (f32, (i32, i32)) {
    let search_coordinates = search_coordinates.unwrap_or_else(|| {
        let (width, height) = unsafe {
            (
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            )
        };
        (pos2(0.0, 0.0), pos2(width as f32, height as f32))
    });

    let screenshot = screenshot_raw(search_coordinates.0, search_coordinates.1)
        .par_chunks(4)
        .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
        .collect::<Vec<_>>();

    let width = (search_coordinates.0.x - search_coordinates.1.x).abs() as usize;
    let height = (search_coordinates.0.y - search_coordinates.1.y).abs() as usize;

    let template = &image.color;

    for _x in 0..=(width - template.width) {
        for _y in 0..=(height - template.height) {
            let mut found = true;
            'lose: {
                for x in 0..template.width {
                    for y in 0..template.height {
                        if screenshot[(_y + y) * width + _x + x]
                            != unsafe {
                                [
                                    *template.pixels.get_unchecked((y * template.width + x) * 4),
                                    *template
                                        .pixels
                                        .get_unchecked((y * template.width + x) * 4 + 1),
                                    *template
                                        .pixels
                                        .get_unchecked((y * template.width + x) * 4 + 2),
                                    *template
                                        .pixels
                                        .get_unchecked((y * template.width + x) * 4 + 3),
                                ]
                            }
                        {
                            found = false;
                            break 'lose;
                        }
                    }
                }
            }
            if found {
                return (
                    1.0,
                    (
                        (_x + template.width / 2) as i32
                            + lesser(search_coordinates.0.x, search_coordinates.1.x) as i32,
                        (_y + template.height / 2) as i32
                            + lesser(search_coordinates.0.y, search_coordinates.1.y) as i32,
                    ),
                );
            }
        }
    }

    (0.0, (0, 0))
}

pub fn get_color_under_mouse() -> Color32 {
    unsafe {
        let dc_screen = GetDC(null_mut());
        let mut point = zeroed();
        GetCursorPos(&mut point);

        let color = GetPixel(dc_screen, point.x, point.y);
        ReleaseDC(null_mut(), dc_screen);

        Color32::from_rgba_premultiplied(GetRValue(color), GetGValue(color), GetBValue(color), 255)
    }
}
