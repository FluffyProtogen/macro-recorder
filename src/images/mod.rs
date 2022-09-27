use egui::{ColorImage, Pos2};
use image::*;
use rayon::{prelude::ParallelIterator, slice::ParallelSliceMut};
use std::ffi::c_void;
use std::ptr::null_mut;
use winapi::um::{wingdi::*, winuser::*};

pub mod image_capture_overlay;

pub struct RawScreenshot {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Clone for RawScreenshot {
    fn clone(&self) -> Self {
        Self {
            pixels: self.pixels.clone(),
            width: self.width,
            height: self.height,
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
        }
    }
}

pub fn screenshot_to_color_image(screenshot: RawScreenshot) -> ColorImage {
    let RawScreenshot {
        pixels: mut pixels_bgra,
        width,
        height,
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

fn find_image() {} // return inputcoordinate x + found x, inputcoordinate y + found y cuz image could be smaller than the whole screen
