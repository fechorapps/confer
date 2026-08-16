use crate::app::ConferApp;
use crate::ui::theme::Theme;
use crate::ui::{captions, chat, controls, diagnostics, polls, roster, watermark, whiteboard};
use egui::{Color32, Pos2, Rect, RichText, Stroke, TextureHandle, Ui, Vec2};

pub fn render_meeting_room(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // 1. Dark Obsidian Canvas Base
    ui.painter().rect_filled(
        full_rect,
        0.0,
        Color32::from_rgb(11, 12, 14), // Obsidian Base
    );

    // 2. Top Header Bar (Elevated Deep Zinc with Precision Bottom Stroke)
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 24))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(0.0)
        .inner_margin(egui::Margin::symmetric(20.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left Brand & Room Information
                ui.horizontal(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(2, 132, 199))
                        .stroke(Stroke::NONE)
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡").size(12.0).color(Color32::WHITE));
                        });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("CONFER")
                            .size(15.0)
                            .strong()
                            .color(Color32::from_rgb(248, 250, 252)),
                    );
                    ui.label(
                        RichText::new("•")
                            .size(10.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                    ui.label(
                        RichText::new(&app.room_title)
                            .size(13.5)
                            .strong()
                            .color(Color32::from_rgb(226, 232, 240)),
                    );
                });

                // Join Code Pill (Click to copy with ephemeral '✓ Copied!' feedback)
                if let Some(code) = &app.current_join_code {
                    ui.add_space(8.0);
                    let now = ui.input(|i| i.time);
                    let is_recently_copied = app
                        .code_copied_time
                        .is_some_and(|t| now - t < 2.0);

                    let (code_text, code_col) = if is_recently_copied {
                        ("✓ Copied!".to_string(), Theme::EMERALD)
                    } else {
                        (format!("CODE: {code}"), Theme::PRIMARY_LIGHT)
                    };

                    let code_btn = ui.add(
                        egui::Button::new(
                            RichText::new(code_text)
                                .size(11.0)
                                .strong()
                                .font(egui::FontId::monospace(11.0))
                                .color(code_col),
                        )
                        .fill(Theme::SURFACE_2)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Theme::RADIUS_SM),
                    );
                    if code_btn
                        .on_hover_text("Click to copy room code to clipboard")
                        .clicked()
                    {
                        ui.output_mut(|o| o.copied_text = code.clone());
                        app.code_copied_time = Some(now);
                    }
                }

                // Room Lock Status Badge
                if app.is_room_locked {
                    ui.add_space(6.0);
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(45, 30, 10))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("🔒 LOCKED")
                                    .size(10.5)
                                    .strong()
                                    .color(Color32::from_rgb(251, 191, 36)),
                            );
                        });
                }

                // Right Status Hub (Clean Telemetry & Streaming Indicators)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // HUD Telemetry Toggle Button
                    let hud_bg = if app.show_diagnostics {
                        crate::ui::theme::Theme::PRIMARY
                    } else {
                        crate::ui::theme::Theme::SURFACE_2
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("⚡ HUD")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(hud_bg)
                            .rounding(crate::ui::theme::Theme::RADIUS_SM),
                        )
                        .on_hover_text("Toggle Real-Time Diagnostics HUD")
                        .clicked()
                    {
                        app.show_diagnostics = !app.show_diagnostics;
                    }

                    ui.add_space(8.0);

                    // Network Health Pill
                    egui::Frame::group(ui.style())
                        .fill(crate::ui::theme::Theme::SURFACE_2)
                        .stroke(Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE))
                        .rounding(crate::ui::theme::Theme::RADIUS_PILL)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("●")
                                        .size(8.0)
                                        .color(crate::ui::theme::Theme::EMERALD),
                                );
                                ui.label(
                                    RichText::new(format!("{}ms RTT", app.rtt_ms))
                                        .size(11.0)
                                        .strong()
                                        .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new("•")
                                        .size(9.0)
                                        .color(crate::ui::theme::Theme::TEXT_MUTED),
                                );
                                ui.label(
                                    RichText::new(format!("{:.1}% Loss", app.packet_loss_pct))
                                        .size(10.5)
                                        .color(crate::ui::theme::Theme::TEXT_SECONDARY),
                                );
                            });
                        });
                });
            });
        });

    // 3. Main Stage & Video Grid + Side Panels
    let available_height = (ui.available_height() - 76.0).max(100.0); // Reserve space for bottom floating dock
    ui.horizontal(|ui| {
        // Stage / Video Grid Container
        ui.vertical(|ui| {
            ui.set_height(available_height);

            if app.is_whiteboard_active {
                whiteboard::render_whiteboard(app, ui);
            } else if app.is_screen_sharing {
                render_screen_share_stage(app, ui);
            } else {
                render_video_grid(app, ui);
            }
        });

        // Optional Side Drawers
        if app.show_chat {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                chat::render_chat(app, ui);
            });
        } else if app.show_roster {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                roster::render_roster(app, ui);
            });
        } else if app.show_polls {
            ui.vertical(|ui| {
                ui.set_width(340.0);
                polls::render_polls(app, ui);
            });
        }
    });

    // 4. Floating Overlay Elements
    render_reactions(app, ui, full_rect);
    render_push_to_talk_indicator(app, ui, full_rect);

    if app.show_diagnostics {
        diagnostics::render_diagnostics(app, ui.ctx());
    }

    captions::render_captions(app, ui, full_rect);

    // 5. Floating Bottom Control Dock
    controls::render_controls(app, ui);

    // 6. Safety Modals
    render_safety_modals(app, ui, full_rect);
}

fn render_push_to_talk_indicator(app: &ConferApp, ui: &mut Ui, full_rect: Rect) {
    if !app.is_push_to_talk_active {
        return;
    }

    let ptt_pos = Pos2::new(full_rect.center().x, full_rect.top() + 64.0);
    let ptt_rect = Rect::from_center_size(ptt_pos, Vec2::new(340.0, 36.0));

    ui.painter().rect_filled(
        ptt_rect,
        18.0,
        Color32::from_rgba_premultiplied(16, 185, 129, 245),
    );
    ui.painter().rect_stroke(
        ptt_rect,
        18.0,
        Stroke::new(1.5_f32, crate::ui::theme::Theme::EMERALD_LIGHT),
    );

    ui.painter().text(
        ptt_pos,
        egui::Align2::CENTER_CENTER,
        "🎙 TRANSMITTING (Push-to-Talk: Spacebar)",
        egui::FontId::proportional(12.5),
        Color32::WHITE,
    );
}

fn render_safety_modals(app: &mut ConferApp, ui: &mut Ui, full_rect: Rect) {
    // Leave Meeting Confirmation Modal
    if app.show_leave_confirmation {
        ui.painter().rect_filled(
            full_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        let modal_w = 400.0_f32;
        let modal_h = 180.0_f32;
        let center = full_rect.center();
        let modal_rect = Rect::from_center_size(center, Vec2::new(modal_w, modal_h));

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(18, 20, 24))
                .stroke(Stroke::new(1.5_f32, Color32::from_rgb(225, 29, 72)))
                .rounding(16.0)
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("⚠️ Leave Meeting?")
                                .size(17.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Are you sure you want to disconnect from this conference room?",
                            )
                            .size(12.5)
                            .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            ui.add_space(32.0);
                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Cancel (Esc)")
                                            .size(12.5)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(38, 42, 48))
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                app.show_leave_confirmation = false;
                            }

                            ui.add_space(16.0);

                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Leave Call (Enter)")
                                            .size(12.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(225, 29, 72))
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                app.show_leave_confirmation = false;
                                app.leave_meeting();
                            }
                        });
                    });
                });
        });
    }

    // Host Kick Participant Confirmation Modal
    if let Some((target_id, target_name)) = app.kick_confirmation_target.clone() {
        ui.painter().rect_filled(
            full_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        let modal_w = 420.0_f32;
        let modal_h = 190.0_f32;
        let center = full_rect.center();
        let modal_rect = Rect::from_center_size(center, Vec2::new(modal_w, modal_h));

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(18, 20, 24))
                .stroke(Stroke::new(1.5_f32, Color32::from_rgb(225, 29, 72)))
                .rounding(16.0)
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Remove Participant?")
                                .size(17.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "Are you sure you want to remove '{target_name}' from this call?"
                            ))
                            .size(12.5)
                            .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            ui.add_space(36.0);
                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Cancel (Esc)")
                                            .size(12.5)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(38, 42, 48))
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                app.kick_confirmation_target = None;
                            }

                            ui.add_space(16.0);

                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Remove (Enter)")
                                            .size(12.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(225, 29, 72))
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                app.host_kick_participant(target_id);
                                app.kick_confirmation_target = None;
                            }
                        });
                    });
                });
        });
    }
}

fn render_reactions(app: &mut ConferApp, ui: &mut Ui, full_rect: Rect) {
    let now = std::time::Instant::now();
    app.active_reactions
        .retain(|r| now.duration_since(r.created_at).as_secs_f32() < 3.0);

    for r in &app.active_reactions {
        let elapsed = now.duration_since(r.created_at).as_secs_f32();
        let y_offset = elapsed * 80.0;
        let x_pos = full_rect.left() + (full_rect.width() * r.x_offset);
        let y_pos = full_rect.bottom() - 100.0 - y_offset;

        let alpha_f = ((1.0 - (elapsed / 3.0)) * 255.0).clamp(0.0, 255.0);
        let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, alpha_f as u8);

        ui.painter().text(
            Pos2::new(x_pos, y_pos),
            egui::Align2::CENTER_CENTER,
            &r.emoji,
            egui::FontId::proportional(28.0),
            text_color,
        );
    }
}

fn render_screen_share_stage(app: &mut ConferApp, ui: &mut Ui) {
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
                                            .color(Color32::WHITE),
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

fn render_video_grid(app: &ConferApp, ui: &mut Ui) {
    let grid_rect = ui.available_rect_before_wrap();
    let count = app.roster.len() + 1;

    let (cols, rows) = match count {
        1 => (1, 1),
        2 => (2, 1),
        3..=4 => (2, 2),
        5..=6 => (3, 2),
        7..=9 => (3, 3),
        _ => (4, (count as f32 / 4.0).ceil() as usize),
    };

    let avail_size = ui.available_size();
    let gap = 12.0_f32;
    let tile_w = ((avail_size.x - (cols as f32 - 1.0) * gap) / cols as f32).max(120.0);
    let tile_h = ((avail_size.y - (rows as f32 - 1.0) * gap) / rows as f32).max(80.0);

    let mut tiles: Vec<TileProps<'_>> = Vec::with_capacity(count);

    // Local participant tile
    tiles.push(TileProps {
        width: tile_w,
        height: tile_h,
        name: &app.user_display_name,
        role: &app.my_role,
        is_audio_muted: app.is_mic_muted,
        is_video_muted: app.is_camera_off,
        is_sharing: app.is_screen_sharing,
        is_hand_raised: app.is_hand_raised,
        is_local: true,
        is_active_speaker: app
            .my_participant_id
            .is_some_and(|id| app.active_speaker_ids.contains(&id)),
        local_texture: app.local_video_texture.as_ref(),
    });

    // Remote participants
    for p in &app.roster {
        tiles.push(TileProps {
            width: tile_w,
            height: tile_h,
            name: &p.display_name,
            role: &p.role,
            is_audio_muted: p.is_audio_muted,
            is_video_muted: p.is_video_muted,
            is_sharing: p.is_screen_sharing,
            is_hand_raised: p.is_hand_raised,
            is_local: false,
            is_active_speaker: app.active_speaker_ids.contains(&p.participant_id),
            local_texture: app.remote_video_textures.get(&p.participant_id),
        });
    }

    egui::Grid::new("video_tiles_grid")
        .spacing([gap, gap])
        .show(ui, |ui| {
            for (idx, props) in tiles.iter().enumerate() {
                render_single_tile(ui, props);

                if (idx + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });

    if app.is_watermark_enabled || app.meeting_policy.watermark_enabled {
        watermark::render_watermark(ui, grid_rect, &app.user_display_name, &app.user_email);
    }
}

pub struct TileProps<'a> {
    pub width: f32,
    pub height: f32,
    pub name: &'a str,
    pub role: &'a str,
    pub is_audio_muted: bool,
    pub is_video_muted: bool,
    pub is_sharing: bool,
    pub is_hand_raised: bool,
    pub is_local: bool,
    pub is_active_speaker: bool,
    pub local_texture: Option<&'a TextureHandle>,
}

fn render_single_tile(ui: &mut Ui, props: &TileProps<'_>) {
    let stroke = if props.is_active_speaker {
        Stroke::new(2.5_f32, crate::ui::theme::Theme::EMERALD) // Emerald active speaker ring
    } else {
        Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE)
    };

    let bg_color = if props.is_video_muted {
        crate::ui::theme::Theme::SURFACE_1
    } else {
        Color32::from_rgb(5, 6, 8) // Deep letterbox canvas
    };

    egui::Frame::group(ui.style())
        .fill(bg_color)
        .stroke(stroke)
        .rounding(crate::ui::theme::Theme::RADIUS_LG)
        .inner_margin(0.0)
        .show(ui, |ui| {
            ui.set_width(props.width);
            ui.set_height(props.height);

            // Active Speaker 2-Pass Ambient Halo
            if props.is_active_speaker {
                let cell_rect = ui.min_rect();
                ui.painter().rect_stroke(
                    cell_rect.expand(2.0),
                    crate::ui::theme::Theme::RADIUS_LG + 2.0,
                    Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(16, 185, 129, 70)),
                );
            }

            if props.is_sharing {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.35);
                    ui.label(
                        RichText::new("🖥 Screen Share Active")
                            .size(15.0)
                            .strong()
                            .color(crate::ui::theme::Theme::PRIMARY_LIGHT),
                    );
                });
            } else if props.is_video_muted {
                ui.vertical_centered(|ui| {
                    ui.add_space((props.height * 0.28).max(10.0));
                    let initial = props
                        .name
                        .chars()
                        .next()
                        .unwrap_or('U')
                        .to_uppercase()
                        .to_string();
                    egui::Frame::group(ui.style())
                        .fill(crate::ui::theme::Theme::SURFACE_2)
                        .stroke(Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE))
                        .rounding(32.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(initial)
                                    .size(28.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                        });
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(props.name)
                            .size(12.5)
                            .strong()
                            .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                    );
                });
            } else if let Some(tex) = props.local_texture {
                // 16:9 Letterbox / Pillarbox Fit (No aspect distortion)
                let target_aspect = 16.0_f32 / 9.0_f32;
                let cell_w = (props.width - 2.0).max(40.0);
                let cell_h = (props.height - 2.0).max(40.0);
                let cell_aspect = cell_w / cell_h;

                let (draw_w, draw_h) = if cell_aspect > target_aspect {
                    (cell_h * target_aspect, cell_h)
                } else {
                    (cell_w, cell_w / target_aspect)
                };

                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::new(tex)
                            .fit_to_exact_size(Vec2::new(draw_w, draw_h))
                            .rounding(crate::ui::theme::Theme::RADIUS_MD),
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.35);
                    ui.label(
                        RichText::new("🎥 Video Stream")
                            .size(13.5)
                            .color(crate::ui::theme::Theme::TEXT_SECONDARY),
                    );
                    if props.is_local {
                        ui.label(
                            RichText::new("(You)")
                                .size(11.0)
                                .color(crate::ui::theme::Theme::TEXT_MUTED),
                        );
                    }
                });
            }

            // Top-Right Raised Hand Pill Badge
            if props.is_hand_raised {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);
                            egui::Frame::group(ui.style())
                                .fill(crate::ui::theme::Theme::AMBER)
                                .stroke(Stroke::NONE)
                                .rounding(crate::ui::theme::Theme::RADIUS_PILL)
                                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("✋ Hand Raised")
                                            .size(10.5)
                                            .strong()
                                            .color(Color32::BLACK),
                                    );
                                });
                        });
                    });
                });
            }

            // Bottom-Left Floating Identity, Mic & Waveform Status Pill
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    egui::Frame::group(ui.style())
                        .fill(crate::ui::theme::Theme::SURFACE_DOCK)
                        .stroke(Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE))
                        .rounding(crate::ui::theme::Theme::RADIUS_SM)
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let mic_icon = if props.is_audio_muted { "🔇" } else { "🎙" };
                                let mic_color = if props.is_audio_muted {
                                    crate::ui::theme::Theme::CRIMSON
                                } else {
                                    crate::ui::theme::Theme::EMERALD
                                };
                                ui.label(RichText::new(mic_icon).size(11.0).color(mic_color));

                                // Active Speaker Audio Waveform Badge for Colorblind Accessibility
                                if props.is_active_speaker && !props.is_audio_muted {
                                    ui.label(
                                        RichText::new("ılı")
                                            .size(11.0)
                                            .strong()
                                            .color(crate::ui::theme::Theme::EMERALD),
                                    );
                                }

                                let name_label = if props.is_local {
                                    format!("{} (You)", props.name)
                                } else {
                                    props.name.to_string()
                                };
                                ui.label(
                                    RichText::new(name_label)
                                        .size(11.5)
                                        .strong()
                                        .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                                );

                                if props.role == "host" {
                                    ui.label(
                                        RichText::new("HOST")
                                            .size(9.0)
                                            .strong()
                                            .color(crate::ui::theme::Theme::PRIMARY_LIGHT),
                                    );
                                }
                            });
                        });
                });
            });
        });
}
