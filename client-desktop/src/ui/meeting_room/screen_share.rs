use crate::app::ConferApp;
use crate::ui::meeting_room::video_grid::{render_single_tile, TileProps};
use crate::ui::theme::Theme;
use crate::ui::watermark;
use egui::{Color32, RichText, Stroke, Ui, Vec2};

pub(super) fn render_screen_share_stage(app: &mut ConferApp, ui: &mut Ui) {
    let avail_w = ui.available_width();
    let avail_h = ui.available_height();
    let show_filmstrip = app.show_stage_filmstrip && avail_w >= 780.0 && !app.roster.is_empty();

    let stage_w = if show_filmstrip {
        (avail_w - 240.0).max(280.0)
    } else {
        avail_w.max(200.0)
    };
    let stage_h = (avail_h - 10.0).max(180.0);

    ui.horizontal(|ui| {
        // Main Screen Share Spotlight Stage
        ui.vertical(|ui| {
            ui.set_width(stage_w);
            ui.set_height(stage_h);

            // Screen Sharing Top Toolbar Banner
            egui::Frame::group(ui.style())
                .fill(Theme::SURFACE_1)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Theme::RADIUS_MD)
                .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let disp_label = if let Some(d) = &app.selected_display {
                            d.label.as_str()
                        } else if app.screen_capturer.picker_mode()
                            == crate::media::PickerMode::Native
                        {
                            "Native Desktop Stream"
                        } else {
                            "Display Screen"
                        };
                        ui.label(RichText::new("🖥️").size(14.0));
                        ui.label(
                            RichText::new(format!("Sharing: {disp_label}"))
                                .size(12.5)
                                .strong()
                                .color(Theme::PRIMARY_LIGHT),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("⏹ Stop Share")
                                            .size(11.5)
                                            .strong()
                                            .color(Theme::ON_ACCENT),
                                    )
                                    .fill(Theme::CRIMSON)
                                    .rounding(Theme::RADIUS_SM),
                                )
                                .clicked()
                            {
                                app.stop_screen_share();
                            }

                            if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native
                                && ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("🔄 Switch Source")
                                                .size(11.5)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Theme::SURFACE_2)
                                        .rounding(Theme::RADIUS_SM),
                                    )
                                    .clicked()
                            {
                                app.stop_screen_share();
                                app.start_native_screen_share();
                            }

                            // Filmstrip Toggle Button
                            if !app.roster.is_empty() && avail_w >= 780.0 {
                                let filmstrip_text = if app.show_stage_filmstrip {
                                    "👥 Hide Filmstrip"
                                } else {
                                    "👥 Show Filmstrip"
                                };
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new(filmstrip_text)
                                                .size(11.0)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Theme::SURFACE_2)
                                        .rounding(Theme::RADIUS_SM),
                                    )
                                    .clicked()
                                {
                                    app.show_stage_filmstrip = !app.show_stage_filmstrip;
                                }
                            }
                        });
                    });
                });

            ui.add_space(8.0);

            // Screen Video Frame Canvas
            egui::Frame::group(ui.style())
                .fill(Color32::BLACK)
                .stroke(Stroke::new(1.5_f32, Theme::PRIMARY))
                .rounding(Theme::RADIUS_LG)
                .inner_margin(0.0)
                .show(ui, |ui| {
                    let display_h = (stage_h - 52.0).max(100.0);
                    ui.set_width(stage_w);
                    ui.set_height(display_h);

                    if let Some(tex) = &app.screen_share_texture {
                        let tex_size = tex.size_vec2();
                        let max_w = (stage_w - 8.0).max(20.0);
                        let max_h = (display_h - 8.0).max(20.0);

                        let aspect = if tex_size.y > 0.0 {
                            tex_size.x / tex_size.y
                        } else {
                            16.0 / 9.0
                        };
                        let mut fit_w = max_w;
                        let mut fit_h = fit_w / aspect;
                        if fit_h > max_h {
                            fit_h = max_h;
                            fit_w = fit_h * aspect;
                        }
                        let fit_w = fit_w.max(10.0);
                        let fit_h = fit_h.max(10.0);

                        ui.centered_and_justified(|ui| {
                            ui.add(
                                egui::Image::new(tex)
                                    .fit_to_exact_size(Vec2::new(fit_w, fit_h))
                                    .rounding(Theme::RADIUS_SM),
                            );
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(display_h * 0.4);
                            ui.label(
                                RichText::new("Initializing screen share stream...")
                                    .size(13.5)
                                    .color(Theme::TEXT_SECONDARY),
                            );
                        });
                    }

                    if app.is_watermark_enabled || app.meeting_policy.watermark_enabled {
                        let stage_rect = ui.min_rect();
                        watermark::render_watermark(
                            ui,
                            stage_rect,
                            &app.user_display_name,
                            &app.user_email,
                        );
                    }
                });
        });

        // Sidebar Strip of Participant Tiles (Conditional on show_filmstrip)
        if show_filmstrip {
            ui.vertical(|ui| {
                ui.set_width(230.0);
                ui.label(
                    RichText::new("Participants")
                        .size(12.0)
                        .strong()
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.add_space(6.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Local Tile
                    let local_props = TileProps {
                        width: 220.0,
                        height: 124.0,
                        name: &app.user_display_name,
                        role: &app.my_role,
                        is_audio_muted: app.is_mic_muted,
                        is_video_muted: app.is_camera_off,
                        is_sharing: app.is_screen_sharing,
                        is_hand_raised: app.is_hand_raised,
                        is_local: true,
                        is_active_speaker: false,
                        local_texture: app.local_video_texture.as_ref(),
                    };
                    render_single_tile(ui, &local_props);

                    ui.add_space(8.0);

                    // Remote Participants
                    for p in &app.roster {
                        let is_active_speaker = app.active_speaker_ids.contains(&p.participant_id);
                        let remote_props = TileProps {
                            width: 220.0,
                            height: 124.0,
                            name: &p.display_name,
                            role: &p.role,
                            is_audio_muted: p.is_audio_muted,
                            is_video_muted: p.is_video_muted,
                            is_sharing: p.is_screen_sharing,
                            is_hand_raised: p.is_hand_raised,
                            is_local: false,
                            is_active_speaker,
                            local_texture: app.remote_video_textures.get(&p.participant_id),
                        };
                        render_single_tile(ui, &remote_props);
                        ui.add_space(8.0);
                    }
                });
            });
        }
    });
}
