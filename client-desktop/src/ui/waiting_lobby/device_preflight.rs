use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui, Vec2};

pub(super) fn render_device_preflight(app: &mut ConferApp, ui: &mut Ui, card_width: f32) {
    // Interactive Pre-Flight Rehearsal Deck (Clickable Toggles)
    ui.label(
        RichText::new("Pre-Flight Device Check")
            .size(12.0)
            .strong()
            .color(Theme::TEXT_PRIMARY),
    );
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        let inner_card_w = (card_width - 32.0).max(100.0);
        let btn_w = ((inner_card_w - 12.0) / 3.0).max(70.0);

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
                Vec2::new(btn_w, 32.0),
                egui::Button::new(
                    RichText::new(cam_text).size(11.0).strong().color(Theme::ON_ACCENT),
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
                Vec2::new(btn_w, 32.0),
                egui::Button::new(
                    RichText::new(mic_text).size(11.0).strong().color(Theme::ON_ACCENT),
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
                Vec2::new(btn_w, 32.0),
                egui::Button::new(
                    RichText::new(denoise_text)
                        .size(11.0)
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
}

pub(super) fn render_mic_vu_meter(app: &ConferApp, ui: &mut Ui, card_width: f32) {
    // Real-Time Live Mic VU Meter Visualizer
    ui.horizontal(|ui| {
        ui.label(RichText::new("Mic Energy:").size(11.0).color(Theme::TEXT_SECONDARY));
        let level = if app.is_mic_muted {
            0.0
        } else {
            app.mic_test_level
        };

        let total_segments = 18;
        let active_segments =
            ((level * total_segments as f32).round() as usize).min(total_segments);
        let segment_gap = 3.0_f32;
        let available_meter_w = (card_width - 120.0).max(40.0);
        let segment_w = ((available_meter_w
            - (total_segments as f32 - 1.0) * segment_gap)
            / total_segments as f32)
            .max(2.0);

        for i in 0..total_segments {
            let is_lit = i < active_segments;
            let seg_color = if !is_lit {
                Theme::SURFACE_2
            } else if i > 14 {
                Color32::from_rgb(244, 63, 94) // Red Peak
            } else if i > 10 {
                Theme::AMBER // Amber Mid
            } else {
                Theme::EMERALD // Emerald Normal
            };
            let seg_rect = ui
                .allocate_exact_size(Vec2::new(segment_w, 7.0), egui::Sense::hover())
                .0;
            ui.painter().rect_filled(seg_rect, 2.0, seg_color);
        }
    });
}
