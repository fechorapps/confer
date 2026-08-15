use egui::Color32;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoFilter {
    #[default]
    None,
    StudioGlow,
    WarmSunset,
    CoolNordic,
    NoirBw,
    VibrantPop,
    VignetteFocus,
    VintageFilm,
}

impl VideoFilter {
    pub fn all() -> &'static [VideoFilter] {
        &[
            VideoFilter::None,
            VideoFilter::StudioGlow,
            VideoFilter::WarmSunset,
            VideoFilter::CoolNordic,
            VideoFilter::NoirBw,
            VideoFilter::VibrantPop,
            VideoFilter::VignetteFocus,
            VideoFilter::VintageFilm,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "Normal",
            Self::StudioGlow => "Studio Glow ✨",
            Self::WarmSunset => "Warm Sunset 🌅",
            Self::CoolNordic => "Nordic Cool ❄️",
            Self::NoirBw => "Noir Cinema 🎬",
            Self::VibrantPop => "Vibrant Pop 🎨",
            Self::VignetteFocus => "Vignette Focus 🎯",
            Self::VintageFilm => "Vintage Film 🎞️",
        }
    }

    pub fn apply(&self, pixels: &mut [Color32], width: usize, height: usize) {
        match self {
            Self::None => {} // Passthrough
            Self::StudioGlow => apply_studio_glow(pixels, width, height),
            Self::WarmSunset => apply_warm_sunset(pixels),
            Self::CoolNordic => apply_cool_nordic(pixels),
            Self::NoirBw => apply_noir_bw(pixels),
            Self::VibrantPop => apply_vibrant_pop(pixels),
            Self::VignetteFocus => apply_vignette_focus(pixels, width, height),
            Self::VintageFilm => apply_vintage_film(pixels),
        }
    }
}

// 1. Studio Glow: Center lighting boost with soft vignette (Multi-core parallel)
fn apply_studio_glow(pixels: &mut [Color32], width: usize, height: usize) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y_f = y as f32;
        let dy = y_f - cy;

        for (x, px) in row.iter_mut().enumerate() {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let center_boost = (1.0 - dist).max(0.0) * 0.25;

            let r = (px.r() as f32 * (1.05 + center_boost) + 6.0).clamp(0.0, 255.0) as u8;
            let g = (px.g() as f32 * (1.05 + center_boost) + 4.0).clamp(0.0, 255.0) as u8;
            let b = (px.b() as f32 * (1.02 + center_boost) + 2.0).clamp(0.0, 255.0) as u8;

            *px = Color32::from_rgb(r, g, b);
        }
    });
}

// 2. Warm Sunset: Golden skin tones & rich shadows (Multi-core parallel)
fn apply_warm_sunset(pixels: &mut [Color32]) {
    pixels.par_iter_mut().for_each(|px| {
        let r = (px.r() as f32 * 1.15 + 12.0).clamp(0.0, 255.0) as u8;
        let g = (px.g() as f32 * 1.05 + 4.0).clamp(0.0, 255.0) as u8;
        let b = (px.b() as f32 * 0.90).clamp(0.0, 255.0) as u8;
        *px = Color32::from_rgb(r, g, b);
    });
}

// 3. Nordic Cool: Modern crisp blue tone (Multi-core parallel)
fn apply_cool_nordic(pixels: &mut [Color32]) {
    pixels.par_iter_mut().for_each(|px| {
        let r = (px.r() as f32 * 0.92).clamp(0.0, 255.0) as u8;
        let g = (px.g() as f32 * 1.02 + 2.0).clamp(0.0, 255.0) as u8;
        let b = (px.b() as f32 * 1.20 + 10.0).clamp(0.0, 255.0) as u8;
        *px = Color32::from_rgb(r, g, b);
    });
}

// 4. Noir Cinema: High contrast black and white (Multi-core parallel)
fn apply_noir_bw(pixels: &mut [Color32]) {
    pixels.par_iter_mut().for_each(|px| {
        let lum = px.r() as f32 * 0.299 + px.g() as f32 * 0.587 + px.b() as f32 * 0.114;
        let contrast_lum = if lum < 128.0 {
            (lum / 128.0).powf(1.2) * 128.0
        } else {
            255.0 - ((255.0 - lum) / 128.0).powf(1.2) * 128.0
        };
        let val = contrast_lum.clamp(0.0, 255.0) as u8;
        *px = Color32::from_rgb(val, val, val);
    });
}

// 5. Vibrant Pop: Vivid saturation boost (Multi-core parallel)
fn apply_vibrant_pop(pixels: &mut [Color32]) {
    pixels.par_iter_mut().for_each(|px| {
        let r = px.r() as f32;
        let g = px.g() as f32;
        let b = px.b() as f32;
        let lum = r * 0.299 + g * 0.587 + b * 0.114;

        let r_pop = (lum + 1.4 * (r - lum)).clamp(0.0, 255.0) as u8;
        let g_pop = (lum + 1.4 * (g - lum)).clamp(0.0, 255.0) as u8;
        let b_pop = (lum + 1.4 * (b - lum)).clamp(0.0, 255.0) as u8;

        *px = Color32::from_rgb(r_pop, g_pop, b_pop);
    });
}

// 6. Vignette Focus: Soft edge darkening (Multi-core parallel)
fn apply_vignette_focus(pixels: &mut [Color32], width: usize, height: usize) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        let y_f = y as f32;
        let dy = y_f - cy;

        for (x, px) in row.iter_mut().enumerate() {
            let dx = x as f32 - cx;
            let dist = (dx * dx + dy * dy).sqrt() / max_dist;
            let factor = (1.0 - dist.powi(2) * 0.6).clamp(0.3, 1.0);

            let r = (px.r() as f32 * factor) as u8;
            let g = (px.g() as f32 * factor) as u8;
            let b = (px.b() as f32 * factor) as u8;

            *px = Color32::from_rgb(r, g, b);
        }
    });
}

// 7. Vintage Film: Sepia analog aesthetic (Multi-core parallel)
fn apply_vintage_film(pixels: &mut [Color32]) {
    pixels.par_iter_mut().for_each(|px| {
        let r = px.r() as f32;
        let g = px.g() as f32;
        let b = px.b() as f32;

        let sepia_r = (r * 0.393 + g * 0.769 + b * 0.189).clamp(0.0, 255.0) as u8;
        let sepia_g = (r * 0.349 + g * 0.686 + b * 0.168).clamp(0.0, 255.0) as u8;
        let sepia_b = (r * 0.272 + g * 0.534 + b * 0.131).clamp(0.0, 255.0) as u8;

        *px = Color32::from_rgb(sepia_r, sepia_g, sepia_b);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_filter_preserves_pixels() {
        let mut pixels = vec![Color32::from_rgb(100, 150, 200); 4];
        let original = pixels.clone();
        VideoFilter::None.apply(&mut pixels, 2, 2);
        assert_eq!(pixels, original);
    }

    #[test]
    fn test_noir_bw_produces_grayscale() {
        let mut pixels = vec![Color32::from_rgb(120, 80, 200)];
        VideoFilter::NoirBw.apply(&mut pixels, 1, 1);
        // All channels in NoirBw must be equal
        assert_eq!(pixels[0].r(), pixels[0].g());
        assert_eq!(pixels[0].g(), pixels[0].b());
    }

    #[test]
    fn test_vintage_film_produces_sepia_tone() {
        let mut pixels = vec![Color32::from_rgb(100, 100, 100)];
        VideoFilter::VintageFilm.apply(&mut pixels, 1, 1);
        // Sepia red component should be highest
        assert!(pixels[0].r() >= pixels[0].g());
        assert!(pixels[0].g() >= pixels[0].b());
    }
}
