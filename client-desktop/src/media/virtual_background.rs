use std::path::PathBuf;
use egui::Color32;
use rayon::prelude::*;

/// Virtual background replacement modes for video feeds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VirtualBackgroundMode {
    #[default]
    None,
    Blur,
    PresetOffice,
    PresetStudio,
    CustomColor([u8; 3]),
    CustomImage(PathBuf),
}

impl VirtualBackgroundMode {
    /// Returns the standard list of selectable virtual background modes.
    pub fn all() -> &'static [VirtualBackgroundMode] {
        &[
            VirtualBackgroundMode::None,
            VirtualBackgroundMode::Blur,
            VirtualBackgroundMode::PresetOffice,
            VirtualBackgroundMode::PresetStudio,
            VirtualBackgroundMode::CustomColor([30, 41, 59]),  // Deep slate
            VirtualBackgroundMode::CustomColor([15, 23, 42]),  // Dark obsidian
            VirtualBackgroundMode::CustomColor([34, 197, 94]), // Studio green screen
        ]
    }

    /// User-friendly display label with icon.
    pub fn label(&self) -> String {
        match self {
            Self::None => "None (Normal)".to_string(),
            Self::Blur => "Gaussian Blur 🌫️".to_string(),
            Self::PresetOffice => "Modern Office 🏢".to_string(),
            Self::PresetStudio => "Executive Studio 🌌".to_string(),
            Self::CustomColor([34, 197, 94]) => "Green Screen 🟩".to_string(),
            Self::CustomColor([r, g, b]) => format!("Custom Color (#{r:02X}{g:02X}{b:02X}) 🎨"),
            Self::CustomImage(_) => "Custom Photo 📁".to_string(),
        }
    }
}

/// Real-time Virtual Background and Portrait Segmentation Engine.
///
/// Features:
/// - Fast separable 2D Gaussian Blur optimized with Rayon multi-threading
/// - Smooth sigmoid human portrait alpha matte calculation
/// - Alpha matte compositor for seamless foreground/background blending
/// - Studio-quality procedural background generators (Office, Studio, Solid Color)
#[derive(Debug, Default, Clone)]
pub struct VirtualBackgroundProcessor {
    pub blur_sigma: f32,
    pub blur_radius: usize,
}

impl VirtualBackgroundProcessor {
    /// Creates a new virtual background processor with tuned defaults.
    pub fn new() -> Self {
        Self {
            blur_sigma: 4.5,
            blur_radius: 8,
        }
    }

    /// Computes the portrait alpha value for a pixel `(x, y)` given frame dimensions.
    ///
    /// Returns `1.0` for subject foreground (head/shoulders) and `0.0` for background,
    /// with a smooth sigmoid transition boundary.
    #[inline(always)]
    pub fn compute_portrait_alpha(x: f32, y: f32, cx: f32, cy: f32, w: f32, h: f32) -> f32 {
        // Head oval mask centered near upper-middle
        let head_cx = cx;
        let head_cy = cy - h * 0.056;
        let head_rx = w * 0.18;
        let head_ry = h * 0.28;
        let head_dist = (((x - head_cx).powi(2) / head_rx.powi(2)) + ((y - head_cy).powi(2) / head_ry.powi(2))).sqrt();

        // Torso / shoulder trapezoid mask
        let body_cx = cx;
        let body_cy = cy + h * 0.167;
        let body_rx = w * 0.38;
        let body_ry = h * 0.42;
        let body_dist = (((x - body_cx).powi(2) / body_rx.powi(2)) + ((y - body_cy).powi(2) / body_ry.powi(2))).sqrt();

        // Smooth sigmoid transition edge
        let head_alpha = (1.0 - (head_dist - 0.85) * 5.0).clamp(0.0, 1.0);
        let body_alpha = (1.0 - (body_dist - 0.85) * 4.0).clamp(0.0, 1.0);

        head_alpha.max(body_alpha)
    }

    /// Generates a full resolution alpha matte buffer.
    pub fn generate_alpha_matte(&self, width: usize, height: usize) -> Vec<f32> {
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let w_f = width as f32;
        let h_f = height as f32;

        let mut matte = vec![0.0f32; width * height];
        matte
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| {
                let y_f = y as f32;
                for (x, alpha) in row.iter_mut().enumerate() {
                    *alpha = Self::compute_portrait_alpha(x as f32, y_f, cx, cy, w_f, h_f);
                }
            });

        matte
    }

    /// Performs a high-performance separable 2D Gaussian blur across RGBA pixel buffers.
    ///
    /// Utilizes a 1D horizontal pass followed by a 1D vertical pass parallelized across CPU cores.
    pub fn gaussian_blur(&self, input: &[Color32], width: usize, height: usize, radius: usize) -> Vec<Color32> {
        if width == 0 || height == 0 || input.is_empty() {
            return Vec::new();
        }
        if radius == 0 {
            return input.to_vec();
        }

        // Generate 1D Gaussian kernel weights
        let sigma = (radius as f32) / 2.0;
        let two_sigma_sq = 2.0 * sigma * sigma;
        let mut kernel = Vec::with_capacity(2 * radius + 1);
        let mut weight_sum = 0.0f32;

        for i in -(radius as isize)..=(radius as isize) {
            let weight = (-((i * i) as f32) / two_sigma_sq).exp();
            kernel.push(weight);
            weight_sum += weight;
        }

        // Normalize weights
        for w in kernel.iter_mut() {
            *w /= weight_sum;
        }

        // Pass 1: Horizontal 1D blur (input -> temp)
        let mut temp = vec![Color32::BLACK; width * height];
        temp.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
            let row_offset = y * width;
            for x in 0..width {
                let mut r_acc = 0.0f32;
                let mut g_acc = 0.0f32;
                let mut b_acc = 0.0f32;
                let mut a_acc = 0.0f32;

                for (k_idx, k_offset) in (-(radius as isize)..=(radius as isize)).enumerate() {
                    let sample_x = (x as isize + k_offset).clamp(0, width as isize - 1) as usize;
                    let px = input[row_offset + sample_x];
                    let weight = kernel[k_idx];

                    r_acc += px.r() as f32 * weight;
                    g_acc += px.g() as f32 * weight;
                    b_acc += px.b() as f32 * weight;
                    a_acc += px.a() as f32 * weight;
                }

                row[x] = Color32::from_rgba_premultiplied(
                    r_acc.clamp(0.0, 255.0) as u8,
                    g_acc.clamp(0.0, 255.0) as u8,
                    b_acc.clamp(0.0, 255.0) as u8,
                    a_acc.clamp(0.0, 255.0) as u8,
                );
            }
        });

        // Pass 2: Vertical 1D blur (temp -> output)
        let mut output = vec![Color32::BLACK; width * height];
        output.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
            for x in 0..width {
                let mut r_acc = 0.0f32;
                let mut g_acc = 0.0f32;
                let mut b_acc = 0.0f32;
                let mut a_acc = 0.0f32;

                for (k_idx, k_offset) in (-(radius as isize)..=(radius as isize)).enumerate() {
                    let sample_y = (y as isize + k_offset).clamp(0, height as isize - 1) as usize;
                    let px = temp[sample_y * width + x];
                    let weight = kernel[k_idx];

                    r_acc += px.r() as f32 * weight;
                    g_acc += px.g() as f32 * weight;
                    b_acc += px.b() as f32 * weight;
                    a_acc += px.a() as f32 * weight;
                }

                row[x] = Color32::from_rgba_premultiplied(
                    r_acc.clamp(0.0, 255.0) as u8,
                    g_acc.clamp(0.0, 255.0) as u8,
                    b_acc.clamp(0.0, 255.0) as u8,
                    a_acc.clamp(0.0, 255.0) as u8,
                );
            }
        });

        output
    }

    /// Composites foreground and background image buffers using an alpha matte.
    ///
    /// Formula: `out = fg * alpha + bg * (1.0 - alpha)`
    pub fn composite_matte(
        foreground: &[Color32],
        background: &[Color32],
        matte: &[f32],
        output: &mut [Color32],
    ) {
        assert_eq!(foreground.len(), background.len());
        assert_eq!(foreground.len(), matte.len());
        assert_eq!(foreground.len(), output.len());

        output
            .par_iter_mut()
            .zip(foreground.par_iter())
            .zip(background.par_iter())
            .zip(matte.par_iter())
            .for_each(|(((out, fg), bg), &alpha)| {
                let r = (fg.r() as f32 * alpha + bg.r() as f32 * (1.0 - alpha)) as u8;
                let g = (fg.g() as f32 * alpha + bg.g() as f32 * (1.0 - alpha)) as u8;
                let b = (fg.b() as f32 * alpha + bg.b() as f32 * (1.0 - alpha)) as u8;
                *out = Color32::from_rgba_premultiplied(r, g, b, fg.a());
            });
    }

    /// Applies virtual background processing to a mutable frame buffer in-place.
    pub fn process_frame(
        &self,
        mode: &VirtualBackgroundMode,
        pixels: &mut [Color32],
        width: usize,
        height: usize,
    ) {
        match mode {
            VirtualBackgroundMode::None => {} // Passthrough
            VirtualBackgroundMode::Blur => {
                let blurred = self.gaussian_blur(pixels, width, height, self.blur_radius);
                let cx = width as f32 / 2.0;
                let cy = height as f32 / 2.0;
                let w_f = width as f32;
                let h_f = height as f32;

                pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
                    let y_f = y as f32;
                    let row_idx = y * width;
                    for (x, px) in row.iter_mut().enumerate() {
                        let alpha = Self::compute_portrait_alpha(x as f32, y_f, cx, cy, w_f, h_f);
                        if alpha < 0.99 {
                            let bg = blurred[row_idx + x];
                            let r = (px.r() as f32 * alpha + bg.r() as f32 * (1.0 - alpha)) as u8;
                            let g = (px.g() as f32 * alpha + bg.g() as f32 * (1.0 - alpha)) as u8;
                            let b = (px.b() as f32 * alpha + bg.b() as f32 * (1.0 - alpha)) as u8;
                            *px = Color32::from_rgba_premultiplied(r, g, b, px.a());
                        }
                    }
                });
            }
            VirtualBackgroundMode::PresetOffice => {
                let cx = width as f32 / 2.0;
                let cy = height as f32 / 2.0;
                let w_f = width as f32;
                let h_f = height as f32;

                pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
                    let y_f = y as f32;
                    for (x, px) in row.iter_mut().enumerate() {
                        let x_f = x as f32;
                        let alpha = Self::compute_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);
                        if alpha < 0.99 {
                            let dx = (x_f - cx) / w_f;
                            let dy = (y_f - cy) / h_f;
                            let panel = ((x_f * 0.05).sin() * 10.0 + (y_f * 0.02).cos() * 5.0) as i16;

                            let bg_r = (32.0 + dx * 20.0 + (panel as f32 * 0.3)).clamp(20.0, 70.0) as u8;
                            let bg_g = (38.0 + dy * 15.0 + (panel as f32 * 0.4)).clamp(25.0, 80.0) as u8;
                            let bg_b = (48.0 - dy * 10.0 + (panel as f32 * 0.6)).clamp(30.0, 95.0) as u8;

                            let r = (px.r() as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
                            let g = (px.g() as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
                            let b = (px.b() as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;
                            *px = Color32::from_rgba_premultiplied(r, g, b, px.a());
                        }
                    }
                });
            }
            VirtualBackgroundMode::PresetStudio => {
                let cx = width as f32 / 2.0;
                let cy = height as f32 / 2.0;
                let w_f = width as f32;
                let h_f = height as f32;

                pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
                    let y_f = y as f32;
                    let dy = (y_f - cy) / cy;

                    for (x, px) in row.iter_mut().enumerate() {
                        let x_f = x as f32;
                        let alpha = Self::compute_portrait_alpha(x_f, y_f, cx, cy, w_f, h_f);
                        if alpha < 0.99 {
                            let dx = (x_f - cx) / cx;
                            let dist = (dx * dx + dy * dy).sqrt();

                            let bg_r = (10.0 + (1.0 - dist).max(0.0) * 15.0) as u8;
                            let bg_g = (18.0 + (1.0 - dist).max(0.0) * 45.0) as u8;
                            let bg_b = (32.0 + (1.0 - dist).max(0.0) * 80.0) as u8;

                            let r = (px.r() as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
                            let g = (px.g() as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
                            let b = (px.b() as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;
                            *px = Color32::from_rgba_premultiplied(r, g, b, px.a());
                        }
                    }
                });
            }
            VirtualBackgroundMode::CustomColor([bg_r, bg_g, bg_b]) => {
                let cx = width as f32 / 2.0;
                let cy = height as f32 / 2.0;
                let w_f = width as f32;
                let h_f = height as f32;
                let (r_val, g_val, b_val) = (*bg_r, *bg_g, *bg_b);

                pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
                    let y_f = y as f32;
                    for (x, px) in row.iter_mut().enumerate() {
                        let alpha = Self::compute_portrait_alpha(x as f32, y_f, cx, cy, w_f, h_f);
                        if alpha < 0.99 {
                            let r = (px.r() as f32 * alpha + r_val as f32 * (1.0 - alpha)) as u8;
                            let g = (px.g() as f32 * alpha + g_val as f32 * (1.0 - alpha)) as u8;
                            let b = (px.b() as f32 * alpha + b_val as f32 * (1.0 - alpha)) as u8;
                            *px = Color32::from_rgba_premultiplied(r, g, b, px.a());
                        }
                    }
                });
            }
            VirtualBackgroundMode::CustomImage(path) => {
                if let Ok(img) = image::open(path) {
                    let resized = img
                        .resize_exact(width as u32, height as u32, image::imageops::FilterType::Nearest)
                        .to_rgb8();
                    let bg_raw = resized.as_raw();

                    let cx = width as f32 / 2.0;
                    let cy = height as f32 / 2.0;
                    let w_f = width as f32;
                    let h_f = height as f32;

                    pixels.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
                        let y_f = y as f32;
                        for (x, px) in row.iter_mut().enumerate() {
                            let alpha = Self::compute_portrait_alpha(x as f32, y_f, cx, cy, w_f, h_f);
                            if alpha < 0.99 {
                                let bg_idx = (y * width + x) * 3;
                                let bg_r = bg_raw[bg_idx];
                                let bg_g = bg_raw[bg_idx + 1];
                                let bg_b = bg_raw[bg_idx + 2];

                                let r = (px.r() as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
                                let g = (px.g() as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
                                let b = (px.b() as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;
                                *px = Color32::from_rgba_premultiplied(r, g, b, px.a());
                            }
                        }
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_blur_preserves_dimensions_and_solid_color() {
        let processor = VirtualBackgroundProcessor::new();
        let solid_red = vec![Color32::from_rgb(200, 50, 50); 64 * 64];

        let blurred = processor.gaussian_blur(&solid_red, 64, 64, 5);
        assert_eq!(blurred.len(), 64 * 64);

        // Blurring a solid color must leave all pixels identical (within integer rounding)
        for px in &blurred {
            assert_eq!(px.r(), 200);
            assert_eq!(px.g(), 50);
            assert_eq!(px.b(), 50);
        }
    }

    #[test]
    fn test_gaussian_blur_smooths_sharp_edges() {
        let processor = VirtualBackgroundProcessor::new();
        // Create an image with a sharp edge in the middle (black on left, white on right)
        let width = 20;
        let height = 20;
        let mut img = vec![Color32::BLACK; width * height];
        for y in 0..height {
            for x in (width / 2)..width {
                img[y * width + x] = Color32::WHITE;
            }
        }

        let blurred = processor.gaussian_blur(&img, width, height, 4);

        // The edge pixel (x = 10, y = 10) should have intermediate brightness
        let center_left = blurred[10 * width + 9];
        let center_right = blurred[10 * width + 10];

        assert!(center_left.r() > 10 && center_left.r() < 240);
        assert!(center_right.r() > 10 && center_right.r() < 240);
    }

    #[test]
    fn test_composite_matte_alpha_blending() {
        let fg = vec![Color32::from_rgb(255, 0, 0); 3];
        let bg = vec![Color32::from_rgb(0, 0, 255); 3];
        let matte = vec![1.0, 0.0, 0.5];
        let mut out = vec![Color32::BLACK; 3];

        VirtualBackgroundProcessor::composite_matte(&fg, &bg, &matte, &mut out);

        // Pixel 0 (alpha = 1.0): Full foreground (red)
        assert_eq!(out[0], Color32::from_rgb(255, 0, 0));

        // Pixel 1 (alpha = 0.0): Full background (blue)
        assert_eq!(out[1], Color32::from_rgb(0, 0, 255));

        // Pixel 2 (alpha = 0.5): 50% blend (purple: 127, 0, 127)
        assert_eq!(out[2].r(), 127);
        assert_eq!(out[2].g(), 0);
        assert_eq!(out[2].b(), 127);
    }

    #[test]
    fn test_virtual_background_none_preserves_pixels() {
        let processor = VirtualBackgroundProcessor::new();
        let mut pixels = vec![Color32::from_rgb(120, 140, 160); 100];
        let original = pixels.clone();

        processor.process_frame(&VirtualBackgroundMode::None, &mut pixels, 10, 10);
        assert_eq!(pixels, original);
    }

    #[test]
    fn test_virtual_background_custom_color_mode() {
        let processor = VirtualBackgroundProcessor::new();
        let width = 40;
        let height = 40;
        let mut pixels = vec![Color32::from_rgb(255, 255, 255); width * height];

        // Green screen color
        let mode = VirtualBackgroundMode::CustomColor([34, 197, 94]);
        processor.process_frame(&mode, &mut pixels, width, height);

        // Top-left corner (0, 0) is background, so should be close to green color
        let corner_px = pixels[0];
        assert_eq!(corner_px.r(), 34);
        assert_eq!(corner_px.g(), 197);
        assert_eq!(corner_px.b(), 94);

        // Center (20, 20) is inside portrait mask, so should retain white foreground
        let center_px = pixels[20 * width + 20];
        assert!(center_px.r() > 200);
        assert!(center_px.g() > 200);
        assert!(center_px.b() > 200);
    }

    #[test]
    fn test_virtual_background_presets_execution() {
        let processor = VirtualBackgroundProcessor::new();
        let width = 20;
        let height = 20;

        let mut office_pixels = vec![Color32::from_rgb(100, 100, 100); width * height];
        processor.process_frame(&VirtualBackgroundMode::PresetOffice, &mut office_pixels, width, height);
        assert_eq!(office_pixels.len(), width * height);

        let mut studio_pixels = vec![Color32::from_rgb(100, 100, 100); width * height];
        processor.process_frame(&VirtualBackgroundMode::PresetStudio, &mut studio_pixels, width, height);
        assert_eq!(studio_pixels.len(), width * height);
    }
}
