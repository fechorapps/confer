use egui::Color32;
use rayon::prelude::*;

/// Converts a tightly-packed RGB24 buffer (3 bytes/pixel, R,G,B order) into
/// `Color32` pixels. Returns `None` if `raw` is smaller than `width * height * 3`
/// bytes, so callers can skip a truncated/corrupt frame instead of panicking.
pub(crate) fn rgb_to_color32(raw: &[u8], width: usize, height: usize) -> Option<Vec<Color32>> {
    if raw.len() < width * height * 3 {
        return None;
    }

    let mut pixels = vec![Color32::BLACK; width * height];
    pixels.par_chunks_mut(width).enumerate().for_each(|(row_y, row_pixels)| {
        let row_offset = row_y * width * 3;
        for (col_x, px) in row_pixels.iter_mut().enumerate() {
            let idx = row_offset + col_x * 3;
            *px = Color32::from_rgb(raw[idx], raw[idx + 1], raw[idx + 2]);
        }
    });
    Some(pixels)
}

/// Converts a tightly-packed BGRx/BGRA32 buffer (4 bytes/pixel, B,G,R,x order,
/// as produced by X11 ZPixmap and SPA's default screen capture format) into
/// `Color32` pixels. Returns `None` if `raw` is smaller than
/// `width * height * 4` bytes.
pub(crate) fn bgrx_to_color32(raw: &[u8], width: usize, height: usize) -> Option<Vec<Color32>> {
    if raw.len() < width * height * 4 {
        return None;
    }

    let mut pixels = vec![Color32::BLACK; width * height];
    pixels.par_chunks_mut(width).enumerate().for_each(|(row_y, row_pixels)| {
        let row_offset = row_y * width * 4;
        for (col_x, px) in row_pixels.iter_mut().enumerate() {
            let idx = row_offset + col_x * 4;
            *px = Color32::from_rgb(raw[idx + 2], raw[idx + 1], raw[idx]);
        }
    });
    Some(pixels)
}
