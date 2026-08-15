use std::path::PathBuf;
use egui::Color32;
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BackgroundEffect {
    #[default]
    None,
    SoftBlur,
    DeepBlur,
    ModernOffice,
    CozyLibrary,
    CyberStudio,
    Custom(PathBuf),
}

impl BackgroundEffect {
    pub fn all() -> &'static [BackgroundEffect] {
        &[
            BackgroundEffect::None,
            BackgroundEffect::SoftBlur,
            BackgroundEffect::DeepBlur,
            BackgroundEffect::ModernOffice,
            BackgroundEffect::CozyLibrary,
            BackgroundEffect::CyberStudio,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "None (Normal)",
            Self::SoftBlur => "Soft Blur 🌫️",
            Self::DeepBlur => "Deep Blur 🌀",
            Self::ModernOffice => "Modern Office 🏢",
            Self::CozyLibrary => "Cozy Library 📚",
            Self::CyberStudio => "Cyber Studio 🌌",
            Self::Custom(_) => "Custom Photo 📁",
        }
    }

    pub fn apply(&self, pixels: &mut [Color32], width: usize, height: usize) {
        match self {
            Self::None => {} // No background modification
            Self::SoftBlur => apply_portrait_blur(pixels, width, height, 4),
            Self::DeepBlur => apply_portrait_blur(pixels, width, height, 10),
            Self::ModernOffice => apply_virtual_office(pixels, width, height),
            Self::CozyLibrary => apply_virtual_library(pixels, width, height),
            Self::CyberStudio => apply_virtual_cyber(pixels, width, height),
            Self::Custom(path) => apply_custom_background(pixels, width, height, path),
        }
    }
}

/// Computes subject portrait alpha matte (1.0 = face/body foreground, 0.0 = background)
#[inline(always)]
fn get_portrait_alpha(x: f32, y: f32, cx: f32, cy: f32, w: f32, h: f32) -> f32 {
    // Center offsets are expressed as fractions of height (tuned at 720p,
    // where 40px ~= 0.056h and 120px ~= 0.167h) rather than absolute pixels,
    // so the mask scales uniformly with capture resolution instead of the
    // head/body centers drifting relative to the (already resolution-relative)
    // radii at non-720p resolutions.
    // 1. Head oval mask (centered near upper center)
    let head_cx = cx;
    let head_cy = cy - h * 0.056;
    let head_rx = w * 0.18;
    let head_ry = h * 0.28;
    let head_dist = ((x - head_cx).powi(2) / head_rx.powi(2) + (y - head_cy).powi(2) / head_ry.powi(2)).sqrt();

    // 2. Torso / Shoulder trapezoid mask
    let body_cx = cx;
    let body_cy = cy + h * 0.167;
    let body_rx = w * 0.38;
    let body_ry = h * 0.42;
    let body_dist = ((x - body_cx).powi(2) / body_rx.powi(2) + (y - body_cy).powi(2) / body_ry.powi(2)).sqrt();

    // Smooth sigmoid transition edge
    let head_alpha = (1.0 - (head_dist - 0.85) * 5.0).clamp(0.0, 1.0);
    let body_alpha = (1.0 - (body_dist - 0.85) * 4.0).clamp(0.0, 1.0);

    head_alpha.max(body_alpha)
}

/// Blends a foreground pixel with a solid background color by `alpha`
/// (1.0 = fully foreground, 0.0 = fully background).
#[inline(always)]
fn blend(fg: Color32, bg_r: u8, bg_g: u8, bg_b: u8, alpha: f32) -> Color32 {
    let r = (fg.r() as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
    let g = (fg.g() as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
    let b = (fg.b() as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;
    Color32::from_rgb(r, g, b)
}

/// Fast multi-core portrait blur (Bokeh Effect)
fn apply_portrait_blur(pixels: &mut [Color32], width: usize, height: usize, blur_radius: usize) {
    let original = pixels.to_vec();
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let w_f = width as f32;
    let h_f = height as f32;
    let step = blur_radius.max(2);

    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y_f = y as f32;
        let y_sample = (y / step) * step;

        for (x, px) in row.iter_mut().enumerate() {
            let x_f = x as f32;
            let alpha = get_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);

            if alpha >= 0.98 {
                continue;
            }

            let x_sample = (x / step) * step;
            let sample_idx = (y_sample.min(height - 1)) * width + (x_sample.min(width - 1));
            let bg_px = original[sample_idx];

            *px = blend(*px, bg_px.r(), bg_px.g(), bg_px.b(), alpha);
        }
    });
}

/// Multi-core Virtual Background: Modern Executive Studio Office
fn apply_virtual_office(pixels: &mut [Color32], width: usize, height: usize) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let w_f = width as f32;
    let h_f = height as f32;

    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y_f = y as f32;

        for (x, px) in row.iter_mut().enumerate() {
            let x_f = x as f32;
            let alpha = get_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);

            if alpha >= 0.98 {
                continue;
            }

            let dx = (x_f - cx) / w_f;
            let dy = (y_f - cy) / h_f;
            let panel_line = ((x_f * 0.05).sin() * 10.0 + (y_f * 0.02).cos() * 5.0) as i16;

            let bg_r = (32.0 + dx * 20.0 + (panel_line as f32 * 0.3)).clamp(20.0, 70.0) as u8;
            let bg_g = (38.0 + dy * 15.0 + (panel_line as f32 * 0.4)).clamp(25.0, 80.0) as u8;
            let bg_b = (48.0 - dy * 10.0 + (panel_line as f32 * 0.6)).clamp(30.0, 95.0) as u8;

            *px = blend(*px, bg_r, bg_g, bg_b, alpha);
        }
    });
}

/// Multi-core Virtual Background: Cozy Modern Library with Bookshelves
fn apply_virtual_library(pixels: &mut [Color32], width: usize, height: usize) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let w_f = width as f32;
    let h_f = height as f32;

    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y_f = y as f32;
        let shelf_y = (y / 80) % 2 == 0;
        let base_wood: f32 = if shelf_y { 55.0 } else { 38.0 };
        let bg_r = (base_wood + 15.0).clamp(25.0, 85.0) as u8;
        let bg_g = (base_wood - 5.0).clamp(20.0, 65.0) as u8;
        let bg_b = (base_wood - 15.0).clamp(15.0, 45.0) as u8;

        for (x, px) in row.iter_mut().enumerate() {
            let x_f = x as f32;
            let alpha = get_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);

            if alpha >= 0.98 {
                continue;
            }

            *px = blend(*px, bg_r, bg_g, bg_b, alpha);
        }
    });
}

/// Multi-core Virtual Background: Cyber Studio (Obsidian & Cyan Glow)
fn apply_virtual_cyber(pixels: &mut [Color32], width: usize, height: usize) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let w_f = width as f32;
    let h_f = height as f32;

    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y_f = y as f32;
        let dy = (y_f - cy) / cy;

        for (x, px) in row.iter_mut().enumerate() {
            let x_f = x as f32;
            let alpha = get_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);

            if alpha >= 0.98 {
                continue;
            }

            let dx = (x_f - cx) / cx;
            let dist = (dx * dx + dy * dy).sqrt();

            let bg_r = (10.0 + (1.0 - dist).max(0.0) * 15.0) as u8;
            let bg_g = (18.0 + (1.0 - dist).max(0.0) * 45.0) as u8;
            let bg_b = (32.0 + (1.0 - dist).max(0.0) * 80.0) as u8;

            *px = blend(*px, bg_r, bg_g, bg_b, alpha);
        }
    });
}

/// Multi-core Custom Image Background Loader & Blending
fn apply_custom_background(pixels: &mut [Color32], width: usize, height: usize, path: &PathBuf) {
    if let Ok(img) = image::open(path) {
        let resized = img.resize_exact(width as u32, height as u32, image::imageops::FilterType::Nearest).to_rgb8();
        let bg_raw = resized.as_raw();

        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let w_f = width as f32;
        let h_f = height as f32;

        pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
            let y_f = y as f32;

            for (x, px) in row.iter_mut().enumerate() {
                let x_f = x as f32;
                let alpha = get_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);

                if alpha >= 0.98 {
                    continue;
                }

                let bg_idx = (y * width + x) * 3;
                let bg_r = bg_raw[bg_idx];
                let bg_g = bg_raw[bg_idx + 1];
                let bg_b = bg_raw[bg_idx + 2];

                *px = blend(*px, bg_r, bg_g, bg_b, alpha);
            }
        });
    }
}
