use crate::app::ConferApp;
use crate::media::filters::VideoFilter;
use crate::media::VirtualBackgroundMode;
use crate::ui::components::Components;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, RichText, Stroke, Ui, Vec2};

/// Renders the Left Column Studio Monitor & Device Controls
pub(super) fn render_studio_viewfinder_card(
    app: &mut ConferApp,
    ui: &mut Ui,
    col_w: f32,
    persona_color: Color32,
    user_initials: &str,
) {
    let card_inner_w = (col_w - 40.0).max(100.0);

    Theme::card_frame(ui.style()).show(ui, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Studio Viewfinder")
                    .size(15.0)
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let status_text = if app.is_camera_off {
                    "📷 Video Paused"
                } else {
                    "● 720p HD • 30 FPS"
                };
                let status_color = if app.is_camera_off {
                    Theme::TEXT_MUTED
                } else {
                    Theme::EMERALD
                };
                let status_bg = if app.is_camera_off {
                    Theme::SURFACE_2
                } else {
                    Color32::from_rgb(6, 78, 59) // Deep Emerald background
                };
                egui::Frame::group(ui.style())
                    .fill(status_bg)
                    .stroke(Stroke::new(
                        1.0_f32,
                        if app.is_camera_off {
                            Theme::BORDER_SUBTLE
                        } else {
                            Theme::EMERALD
                        },
                    ))
                    .rounding(Theme::RADIUS_PILL)
                    .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(status_text)
                                .size(10.5)
                                .strong()
                                .color(status_color),
                        );
                    });
            });
        });

        ui.add_space(12.0);

        // 16:9 Video Canvas Surface with Studio Lens Reticle
        let viewport_h = (card_inner_w * 0.5625).max(140.0);

        let canvas_response = egui::Frame::group(ui.style())
            .fill(Theme::CANVAS)
            .stroke(Stroke::new(1.5_f32, Theme::SURFACE_3))
            .rounding(Theme::RADIUS_MD)
            .inner_margin(0.0)
            .show(ui, |ui| {
                ui.set_width(card_inner_w);
                ui.set_height(viewport_h);

                if !app.is_camera_off {
                    if let Some(tex) = &app.local_video_texture {
                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::Image::new(tex)
                                    .fit_to_exact_size(Vec2::new(card_inner_w, viewport_h))
                                    .rounding(Theme::RADIUS_MD),
                            );
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(viewport_h * 0.28);
                        Components::avatar_badge(ui, user_initials, 56.0, 22.0, persona_color);
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Camera is paused")
                                .size(12.5)
                                .color(Theme::TEXT_SECONDARY),
                        );
                    });
                }
            });

        // Overlay Subtle Studio Viewfinder Corner Reticles
        let cr = canvas_response.response.rect;
        let p = ui.painter();
        let reticle_color = Color32::from_rgba_premultiplied(255, 255, 255, 45);
        let reticle_stroke = Stroke::new(1.5_f32, reticle_color);
        let r_len = 12.0_f32;
        let r_pad = 8.0_f32;

        // Top-Left
        p.line_segment(
            [
                Pos2::new(cr.min.x + r_pad, cr.min.y + r_pad),
                Pos2::new(cr.min.x + r_pad + r_len, cr.min.y + r_pad),
            ],
            reticle_stroke,
        );
        p.line_segment(
            [
                Pos2::new(cr.min.x + r_pad, cr.min.y + r_pad),
                Pos2::new(cr.min.x + r_pad, cr.min.y + r_pad + r_len),
            ],
            reticle_stroke,
        );
        // Top-Right
        p.line_segment(
            [
                Pos2::new(cr.max.x - r_pad, cr.min.y + r_pad),
                Pos2::new(cr.max.x - r_pad - r_len, cr.min.y + r_pad),
            ],
            reticle_stroke,
        );
        p.line_segment(
            [
                Pos2::new(cr.max.x - r_pad, cr.min.y + r_pad),
                Pos2::new(cr.max.x - r_pad, cr.min.y + r_pad + r_len),
            ],
            reticle_stroke,
        );
        // Bottom-Left
        p.line_segment(
            [
                Pos2::new(cr.min.x + r_pad, cr.max.y - r_pad),
                Pos2::new(cr.min.x + r_pad + r_len, cr.max.y - r_pad),
            ],
            reticle_stroke,
        );
        p.line_segment(
            [
                Pos2::new(cr.min.x + r_pad, cr.max.y - r_pad),
                Pos2::new(cr.min.x + r_pad, cr.max.y - r_pad - r_len),
            ],
            reticle_stroke,
        );
        // Bottom-Right
        p.line_segment(
            [
                Pos2::new(cr.max.x - r_pad, cr.max.y - r_pad),
                Pos2::new(cr.max.x - r_pad - r_len, cr.max.y - r_pad),
            ],
            reticle_stroke,
        );
        p.line_segment(
            [
                Pos2::new(cr.max.x - r_pad, cr.max.y - r_pad),
                Pos2::new(cr.max.x - r_pad, cr.max.y - r_pad - r_len),
            ],
            reticle_stroke,
        );

        ui.add_space(14.0);

        // Hardware Toggles with Colorized States & Hotkey Hints
        ui.horizontal(|ui| {
            let btn_w = ((card_inner_w - 12.0) / 3.0).max(80.0);

            // Camera Toggle
            let cam_bg = if app.is_camera_off {
                Theme::CRIMSON
            } else {
                Theme::EMERALD
            };
            let cam_text = if app.is_camera_off {
                "📷 Cam Off"
            } else {
                "🎥 Cam Active"
            };
            if ui
                .add_sized(
                    Vec2::new(btn_w, 34.0),
                    egui::Button::new(
                        RichText::new(cam_text)
                            .size(11.5)
                            .strong()
                            .color(Theme::ON_ACCENT),
                    )
                    .fill(cam_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .on_hover_text("Toggle Camera (Ctrl+E)")
                .clicked()
            {
                app.toggle_camera();
            }

            // Mic Toggle
            let mic_bg = if app.is_mic_muted {
                Theme::CRIMSON
            } else {
                Theme::EMERALD
            };
            let mic_text = if app.is_mic_muted {
                "🔇 Mic Muted"
            } else {
                "🎙 Mic Active"
            };
            if ui
                .add_sized(
                    Vec2::new(btn_w, 34.0),
                    egui::Button::new(
                        RichText::new(mic_text)
                            .size(11.5)
                            .strong()
                            .color(Theme::ON_ACCENT),
                    )
                    .fill(mic_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .on_hover_text("Toggle Microphone (Ctrl+D)")
                .clicked()
            {
                app.toggle_mic();
            }

            // AI Denoise Toggle
            let denoise_bg = if app.is_ai_denoise_enabled {
                Theme::PRIMARY
            } else {
                Theme::SURFACE_2
            };
            let denoise_text = if app.is_ai_denoise_enabled {
                "⚡ Denoise ON"
            } else {
                "⚡ Denoise OFF"
            };
            let denoise_fg = if app.is_ai_denoise_enabled {
                Theme::ON_ACCENT
            } else {
                Color32::WHITE
            };
            if ui
                .add_sized(
                    Vec2::new(btn_w, 34.0),
                    egui::Button::new(
                        RichText::new(denoise_text)
                            .size(11.5)
                            .strong()
                            .color(denoise_fg),
                    )
                    .fill(denoise_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .on_hover_text("RNNoise 48kHz Neural Noise Suppression")
                .clicked()
            {
                app.toggle_ai_denoise();
            }
        });

        ui.add_space(14.0);

        // Responsive Studio Audio VU Visualizer (3-Tier High-Contrast Color Bands)
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Mic Energy:")
                    .size(11.0)
                    .color(Theme::TEXT_SECONDARY),
            );
            let level = if app.is_mic_muted {
                0.0
            } else {
                app.mic_test_level
            };

            let total_segments = 20;
            let active_segments =
                ((level * total_segments as f32).round() as usize).min(total_segments);
            let segment_gap = 3.0_f32;
            let available_meter_w = (card_inner_w - 85.0).max(40.0);
            let segment_w = ((available_meter_w - (total_segments as f32 - 1.0) * segment_gap)
                / total_segments as f32)
                .max(2.0);

            for i in 0..total_segments {
                let is_lit = i < active_segments;
                let seg_color = if !is_lit {
                    Theme::SURFACE_2
                } else if i > 16 {
                    Color32::from_rgb(244, 63, 94) // Red Peak (#F43F5E)
                } else if i > 12 {
                    Theme::AMBER // Amber Mid-High (#F59E0B)
                } else {
                    Theme::EMERALD // Emerald Normal (#10B981)
                };
                let seg_rect = ui
                    .allocate_exact_size(Vec2::new(segment_w, 8.0), egui::Sense::hover())
                    .0;
                ui.painter().rect_filled(seg_rect, 2.0, seg_color);
            }
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        // Visual Tone Filters with Wrapped Grid (No horizontal scroll friction)
        ui.label(
            RichText::new("Color Tone Preset")
                .size(12.0)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for filter in VideoFilter::all() {
                let is_active = app.active_filter == *filter;
                let (swatch_color, bg) = if is_active {
                    (Color32::WHITE, Theme::PRIMARY)
                } else {
                    (Theme::TEXT_SECONDARY, Theme::SURFACE_2)
                };
                let text_color = if is_active {
                    Theme::ON_ACCENT
                } else {
                    Color32::from_rgb(203, 213, 225)
                };

                let dot = match filter {
                    VideoFilter::StudioGlow => "✨ ",
                    VideoFilter::WarmSunset => "🟠 ",
                    VideoFilter::CoolNordic => "🔵 ",
                    VideoFilter::NoirBw => "⚪ ",
                    VideoFilter::VibrantPop => "🟣 ",
                    VideoFilter::VignetteFocus => "🎯 ",
                    VideoFilter::VintageFilm => "🟤 ",
                    VideoFilter::None => "🌿 ",
                };
                let label = format!("{}{}", dot, filter.label());

                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(label).size(11.0).strong().color(text_color),
                        )
                        .fill(bg)
                        .stroke(Stroke::new(
                            1.0_f32,
                            if is_active {
                                swatch_color
                            } else {
                                Theme::BORDER_SUBTLE
                            },
                        ))
                        .rounding(Theme::RADIUS_PILL),
                    )
                    .clicked()
                {
                    app.active_filter = *filter;
                }
            }
        });

        ui.add_space(12.0);

        // Background & Portrait Blur Selector (Wrapped Grid)
        ui.label(
            RichText::new("Studio Background & Blur")
                .size(12.0)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for bg in VirtualBackgroundMode::all() {
                let is_active = app.virtual_bg_mode == *bg;
                let btn_bg = if is_active {
                    Theme::PRIMARY
                } else {
                    Theme::SURFACE_2
                };
                let text_color = if is_active {
                    Theme::ON_ACCENT
                } else {
                    Color32::from_rgb(203, 213, 225)
                };
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(bg.label())
                                .size(11.0)
                                .strong()
                                .color(text_color),
                        )
                        .fill(btn_bg)
                        .rounding(Theme::RADIUS_PILL),
                    )
                    .clicked()
                {
                    app.set_virtual_bg_mode(bg.clone());
                }
            }

            let is_custom = matches!(app.virtual_bg_mode, VirtualBackgroundMode::CustomImage(_));
            let custom_bg = if is_custom {
                Theme::PRIMARY
            } else {
                Theme::SURFACE_2
            };
            let custom_text_color = if is_custom {
                Theme::ON_ACCENT
            } else {
                Color32::WHITE
            };
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("📁 Custom Photo...")
                            .size(11.0)
                            .strong()
                            .color(custom_text_color),
                    )
                    .fill(custom_bg)
                    .rounding(Theme::RADIUS_PILL),
                )
                .clicked()
            {
                app.choose_custom_background();
            }
        });
    });
}
