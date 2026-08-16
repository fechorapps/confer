use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui, Vec2};

pub(super) fn render_media_cluster(app: &mut ConferApp, ui: &mut Ui) {
    let mic_bg = if app.is_push_to_talk_active {
        Theme::EMERALD
    } else if app.is_mic_muted {
        Theme::SURFACE_2
    } else {
        Theme::SURFACE_3
    };

    let level = if app.is_mic_muted { 0.0 } else { app.mic_test_level };
    let mic_text = if app.is_push_to_talk_active {
        "🎙 PTT Active"
    } else if app.is_mic_muted {
        "🔇 Mic Off"
    } else if level > 0.35 {
        "🎙 Mic On ▮▮▮"
    } else if level > 0.12 {
        "🎙 Mic On ▮▮▯"
    } else {
        "🎙 Mic On ▮▯▯"
    };

    let mic_hover = if app.is_mic_muted {
        "Microphone Muted (Click or Ctrl+D to unmute, hold Spacebar for PTT)"
    } else {
        "Microphone Active (Click or Ctrl+D to mute)"
    };

    let mic_text_col = if app.is_mic_muted {
        Theme::TEXT_SECONDARY
    } else {
        Color32::WHITE
    };

    if ui
        .add_sized(
            Vec2::new(0.0, 32.0),
            egui::Button::new(
                RichText::new(mic_text)
                    .size(11.5)
                    .strong()
                    .color(mic_text_col),
            )
            .fill(mic_bg)
            .rounding(Theme::RADIUS_PILL),
        )
        .on_hover_text(mic_hover)
        .clicked()
    {
        app.toggle_mic();
    }

    // Camera Toggle
    let cam_bg = if app.is_camera_off {
        Theme::SURFACE_2
    } else {
        Theme::SURFACE_3
    };
    let cam_text = if app.is_camera_off {
        "📷 Cam Off"
    } else {
        "🎥 Cam On"
    };
    let cam_text_col = if app.is_camera_off {
        Theme::TEXT_SECONDARY
    } else {
        Color32::WHITE
    };

    if ui
        .add_sized(
            Vec2::new(0.0, 32.0),
            egui::Button::new(
                RichText::new(cam_text)
                    .size(11.5)
                    .strong()
                    .color(cam_text_col),
            )
            .fill(cam_bg)
            .rounding(Theme::RADIUS_PILL),
        )
        .on_hover_text("Toggle Camera (Ctrl+E)")
        .clicked()
    {
        app.toggle_camera();
    }

    // Screen Share Control
    if app.is_screen_sharing {
        if ui
            .add_sized(
                Vec2::new(0.0, 32.0),
                egui::Button::new(
                    RichText::new("⏹ Stop Share")
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Theme::PRIMARY)
                .rounding(Theme::RADIUS_PILL),
            )
            .on_hover_text("Stop Screen Sharing (Ctrl+Shift+S)")
            .clicked()
        {
            app.stop_screen_share();
        }
    } else if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
        if ui
            .add_sized(
                Vec2::new(0.0, 32.0),
                egui::Button::new(
                    RichText::new("🖥 Share")
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Theme::SURFACE_2)
                .rounding(Theme::RADIUS_PILL),
            )
            .on_hover_text("Share Screen via Portal (Ctrl+Shift+S)")
            .clicked()
        {
            app.start_native_screen_share();
        }
    } else {
        let mut selected_display: Option<usize> = None;
        ui.menu_button(
            RichText::new("🖥 Share").size(11.5).strong().color(Color32::WHITE),
            |ui| {
                ui.set_min_width(220.0);
                ui.label(
                    RichText::new("Select Display to Share")
                        .size(11.0)
                        .strong()
                        .color(Theme::PRIMARY_LIGHT),
                );
                ui.separator();
                for (idx, d) in app.available_displays.iter().enumerate() {
                    if ui
                        .button(RichText::new(&d.label).size(11.0).color(Color32::WHITE))
                        .clicked()
                    {
                        selected_display = Some(idx);
                        ui.close_menu();
                    }
                }
            },
        );
        if let Some(idx) = selected_display {
            if let Some(d) = app.available_displays.get(idx).cloned() {
                app.start_screen_share(d);
            }
        }
    }
}
