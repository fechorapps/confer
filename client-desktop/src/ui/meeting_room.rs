use egui::{Color32, Pos2, Rect, RichText, Stroke, TextureHandle, Ui, Vec2};
use uuid::Uuid;
use crate::app::ConferApp;
use crate::ui::{captions, chat, controls, diagnostics, polls, roster, watermark, whiteboard};


pub fn render_meeting_room(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // Dark Obsidian canvas background
    ui.painter().rect_filled(full_rect, 0.0, Color32::from_rgb(11, 12, 14));

    // Top Header Bar
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 23))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(0.0)
        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("CONFER").size(15.0).strong().color(Color32::from_rgb(56, 189, 248)));
                ui.label(RichText::new("•").color(Color32::from_rgb(71, 85, 105)));
                ui.label(RichText::new(&app.room_title).size(14.0).strong().color(Color32::from_rgb(248, 250, 252)));

                if let Some(code) = &app.current_join_code {
                    ui.add_space(8.0);
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(26, 29, 33))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("CODE: {code}")).size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                            });
                        });
                }

                if app.is_room_locked {
                    ui.add_space(6.0);
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(45, 30, 10))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(245, 158, 11)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("🔒 LOCKED").size(11.0).strong().color(Color32::from_rgb(251, 191, 36)));
                        });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let chat_btn_text = if app.unread_chat_count > 0 {
                        format!("💬 Chat ({} unread)", app.unread_chat_count)
                    } else {
                        "💬 Chat".to_string()
                    };
                    let chat_bg = if app.show_chat { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new(chat_btn_text).size(12.0).color(Color32::WHITE)).fill(chat_bg).rounding(6.0)).clicked() {
                        app.show_chat = !app.show_chat;
                        if app.show_chat {
                            app.show_roster = false;
                            app.show_polls = false;
                            app.unread_chat_count = 0;
                        }
                    }

                    let roster_bg = if app.show_roster { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new(format!("👥 People ({})", app.roster.len() + 1)).size(12.0).color(Color32::WHITE)).fill(roster_bg).rounding(6.0)).clicked() {
                        app.show_roster = !app.show_roster;
                        if app.show_roster {
                            app.show_chat = false;
                            app.show_polls = false;
                        }
                    }

                    let polls_bg = if app.show_polls { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new("📊 Polls").size(12.0).color(Color32::WHITE)).fill(polls_bg).rounding(6.0)).clicked() {
                        app.show_polls = !app.show_polls;
                        if app.show_polls {
                            app.show_chat = false;
                            app.show_roster = false;
                        }
                    }

                    let diag_bg = if app.show_diagnostics { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new("⚙ Health HUD").size(12.0).color(Color32::WHITE)).fill(diag_bg).rounding(6.0)).clicked() {
                        app.show_diagnostics = !app.show_diagnostics;
                    }
                });
            });
        });

    // Main Stage & Video Grid + Side Panels
    let available_height = ui.available_height() - 76.0; // Space for bottom floating dock
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

    // Floating Emoji Reaction Animations
    render_reactions(app, ui, full_rect);

    // Push-to-Talk Active Glow Banner
    render_push_to_talk_indicator(app, ui, full_rect);

    // Diagnostics HUD Modal / Window
    if app.show_diagnostics {
        diagnostics::render_diagnostics(app, ui.ctx());
    }

    // Floating Live Captions Overlay
    captions::render_captions(app, ui, full_rect);

    // Bottom Control Dock
    controls::render_controls(app, ui);

    // Destructive Actions Safety Confirmation Modals (Leave Meeting & Kick Participant)
    render_safety_modals(app, ui, full_rect);
}

fn render_push_to_talk_indicator(app: &ConferApp, ui: &mut Ui, full_rect: Rect) {
    if !app.is_push_to_talk_active {
        return;
    }

    let ptt_pos = Pos2::new(full_rect.center().x, full_rect.top() + 64.0);
    let ptt_rect = Rect::from_center_size(ptt_pos, Vec2::new(340.0, 36.0));

    ui.painter().rect_filled(ptt_rect, 18.0, Color32::from_rgba_premultiplied(16, 185, 129, 245));
    ui.painter().rect_stroke(ptt_rect, 18.0, Stroke::new(1.5_f32, Color32::from_rgb(209, 250, 229)));

    ui.painter().text(
        ptt_pos,
        egui::Align2::CENTER_CENTER,
        "🎙 TRANSMITIENDO (Push-to-Talk: Espacio)",
        egui::FontId::proportional(12.5),
        Color32::WHITE,
    );
}

fn render_safety_modals(app: &mut ConferApp, ui: &mut Ui, full_rect: Rect) {
    let ctx = ui.ctx().clone();

    // 1. Leave Meeting Confirmation Modal
    if app.show_leave_confirmation {
        // Dim background overlay
        ui.painter().rect_filled(full_rect, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 180));

        let modal_pos = full_rect.center();
        let modal_size = Vec2::new(380.0, 180.0);
        let modal_rect = Rect::from_center_size(modal_pos, modal_size);

        egui::Area::new(egui::Id::new("leave_confirm_modal"))
            .fixed_pos(modal_rect.min)
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(18, 20, 23))
                    .stroke(Stroke::new(1.5_f32, Color32::from_rgb(225, 29, 72)))
                    .rounding(16.0)
                    .inner_margin(egui::Margin::symmetric(24.0, 20.0))
                    .show(ui, |ui| {
                        ui.set_width(modal_size.x - 48.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("¿Deseas salir de la reunión?").size(16.0).strong().color(Color32::WHITE));
                            ui.add_space(8.0);
                            ui.label(RichText::new("Se interrumpirá tu conexión de audio y video con los participantes.").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(20.0);

                            ui.horizontal(|ui| {
                                ui.columns(2, |cols| {
                                    if cols[0].add_sized(Vec2::new(cols[0].available_width(), 34.0), egui::Button::new(RichText::new("Cancelar (Esc)").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(38, 42, 48)).rounding(8.0)).clicked() {
                                        app.show_leave_confirmation = false;
                                    }
                                    if cols[1].add_sized(Vec2::new(cols[1].available_width(), 34.0), egui::Button::new(RichText::new("✕ Salir de la Sala").size(12.0).strong().color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(8.0)).clicked() {
                                        app.leave_meeting();
                                    }
                                });
                            });
                        });
                    });
            });
    }

    // 2. Kick Participant Confirmation Modal
    if let Some((target_id, target_name)) = app.kick_confirmation_target.clone() {
        // Dim background overlay
        ui.painter().rect_filled(full_rect, 0.0, Color32::from_rgba_premultiplied(0, 0, 0, 180));

        let modal_pos = full_rect.center();
        let modal_size = Vec2::new(400.0, 190.0);
        let modal_rect = Rect::from_center_size(modal_pos, modal_size);

        egui::Area::new(egui::Id::new("kick_confirm_modal"))
            .fixed_pos(modal_rect.min)
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(18, 20, 23))
                    .stroke(Stroke::new(1.5_f32, Color32::from_rgb(225, 29, 72)))
                    .rounding(16.0)
                    .inner_margin(egui::Margin::symmetric(24.0, 20.0))
                    .show(ui, |ui| {
                        ui.set_width(modal_size.x - 48.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(format!("¿Expulsar a {target_name}?")).size(16.0).strong().color(Color32::WHITE));
                            ui.add_space(8.0);
                            ui.label(RichText::new("El participante será desconectado inmediatamente de la conferencia.").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(20.0);

                            ui.horizontal(|ui| {
                                ui.columns(2, |cols| {
                                    if cols[0].add_sized(Vec2::new(cols[0].available_width(), 34.0), egui::Button::new(RichText::new("Cancelar (Esc)").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(38, 42, 48)).rounding(8.0)).clicked() {
                                        app.kick_confirmation_target = None;
                                    }
                                    if cols[1].add_sized(Vec2::new(cols[1].available_width(), 34.0), egui::Button::new(RichText::new("🚫 Expulsar").size(12.0).strong().color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(8.0)).clicked() {
                                        app.host_kick_participant(target_id);
                                        app.kick_confirmation_target = None;
                                    }
                                });
                            });
                        });
                    });
            });
    }
}


/// Renders the Presentation Stage when Screen Sharing is active
fn render_screen_share_stage(app: &mut ConferApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let stage_w = if app.roster.is_empty() { ui.available_width() } else { ui.available_width() - 240.0 };
        let stage_h = ui.available_height() - 10.0;

        // Big Screen Presentation Frame
        ui.vertical(|ui| {
            ui.set_width(stage_w);
            ui.set_height(stage_h);

            // Screen Sharing Header Banner
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(18, 20, 23))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let disp_label = if let Some(d) = &app.selected_display {
                            d.label.as_str()
                        } else if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
                            "Native Screen / Window"
                        } else {
                            "Display"
                        };
                        ui.label(RichText::new(format!("🖥️ Sharing: {disp_label}")).size(12.0).strong().color(Color32::from_rgb(56, 189, 248)));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("⏹ Stop Sharing").size(11.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(6.0)).clicked() {
                                app.stop_screen_share();
                            }

                            // Switch Display Button / Dropdown
                            if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
                                if ui.add(egui::Button::new(RichText::new("🔄 Switch Source").size(11.0).color(Color32::WHITE)).fill(Color32::from_rgb(26, 29, 33)).rounding(6.0)).clicked() {
                                    app.stop_screen_share();
                                    app.start_native_screen_share();
                                }
                            } else {
                                ui.menu_button(RichText::new("🔄 Switch Display").size(11.0).color(Color32::WHITE), |ui| {
                                    ui.set_min_width(200.0);
                                    let displays = app.available_displays.clone();
                                    for d in displays {
                                        if ui.button(RichText::new(&d.label).size(11.0).color(Color32::WHITE)).clicked() {
                                            app.start_screen_share(d);
                                            ui.close_menu();
                                        }
                                    }
                                });
                            }
                        });
                    });
                });

            ui.add_space(8.0);

            // Live Screen Surface Frame
            egui::Frame::group(ui.style())
                .fill(Color32::BLACK)
                .stroke(Stroke::new(1.5_f32, Color32::from_rgb(2, 132, 199)))
                .rounding(10.0)
                .inner_margin(0.0)
                .show(ui, |ui| {
                    let display_h = stage_h - 48.0;
                    ui.set_width(stage_w);
                    ui.set_height(display_h);

                    if let Some(tex) = &app.screen_share_texture {
                        ui.centered_and_justified(|ui| {
                            ui.image((tex.id(), Vec2::new(stage_w - 4.0, display_h - 4.0)));
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(display_h * 0.4);
                            ui.label(RichText::new("Initializing screen share stream...").size(14.0).color(Color32::from_rgb(148, 163, 184)));
                        });
                    }

                    if app.is_watermark_enabled || app.meeting_policy.watermark_enabled {
                        let stage_rect = ui.min_rect();
                        watermark::render_watermark(ui, stage_rect, &app.user_display_name, &app.user_email);
                    }
                });
        });

        // Sidebar of participant tiles
        ui.vertical(|ui| {
            ui.set_width(230.0);
            ui.label(RichText::new("Participants").size(12.0).strong().color(Color32::from_rgb(148, 163, 184)));
            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Local tile
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

                // Remote participant tiles
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
                        local_texture: None,
                    };
                    render_single_tile(ui, &remote_props);
                    ui.add_space(8.0);
                }
            });
        });
    });
}

fn render_video_grid(app: &ConferApp, ui: &mut Ui) {
    let grid_rect = ui.available_rect_before_wrap();
    let mut total_participants = Vec::new();

    // Local participant tile
    total_participants.push((
        app.my_participant_id.unwrap_or(Uuid::nil()),
        app.user_display_name.clone(),
        app.my_role.clone(),
        app.is_mic_muted,
        app.is_camera_off,
        app.is_screen_sharing,
        app.is_hand_raised,
        true // is_local
    ));

    // Remote participants
    for p in &app.roster {
        total_participants.push((
            p.participant_id,
            p.display_name.clone(),
            p.role.clone(),
            p.is_audio_muted,
            p.is_video_muted,
            p.is_screen_sharing,
            p.is_hand_raised,
            false // is_local
        ));
    }

    let count = total_participants.len();
    let (cols, rows) = match count {
        1 => (1, 1),
        2 => (2, 1),
        3..=4 => (2, 2),
        5..=6 => (3, 2),
        7..=9 => (3, 3),
        _ => (4, (count as f32 / 4.0).ceil() as usize),
    };

    let avail_size = ui.available_size();
    let tile_w = (avail_size.x / cols as f32) - 10.0;
    let tile_h = (avail_size.y / rows as f32) - 10.0;

    egui::Grid::new("video_tiles_grid")
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            for (idx, (p_id, name, role, is_audio_muted, is_video_muted, is_sharing, is_hand_raised, is_local)) in total_participants.iter().enumerate() {
                let is_active_speaker = app.active_speaker_ids.contains(p_id);

                let props = TileProps {
                    width: tile_w,
                    height: tile_h,
                    name,
                    role,
                    is_audio_muted: *is_audio_muted,
                    is_video_muted: *is_video_muted,
                    is_sharing: *is_sharing,
                    is_hand_raised: *is_hand_raised,
                    is_local: *is_local,
                    is_active_speaker,
                    local_texture: app.local_video_texture.as_ref(),
                };

                render_single_tile(ui, &props);

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
        Stroke::new(2.5_f32, Color32::from_rgb(16, 185, 129)) // Emerald speaker ring
    } else {
        Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44))
    };

    let bg_color = if props.is_video_muted {
        Color32::from_rgb(18, 20, 23)
    } else {
        Color32::from_rgb(11, 12, 14)
    };

    egui::Frame::group(ui.style())
        .fill(bg_color)
        .stroke(stroke)
        .rounding(10.0)
        .inner_margin(0.0)
        .show(ui, |ui| {
            ui.set_width(props.width);
            ui.set_height(props.height);

            if props.is_sharing {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.35);
                    ui.label(RichText::new("🖥 Screen Share Active").size(16.0).strong().color(Color32::from_rgb(56, 189, 248)));
                });
            } else if props.is_video_muted {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.28);
                    let initial = props.name.chars().next().unwrap_or('U').to_uppercase().to_string();
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(38, 42, 48))
                        .stroke(Stroke::NONE)
                        .rounding(32.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new(initial).size(30.0).strong().color(Color32::WHITE));
                        });
                    ui.add_space(8.0);
                    ui.label(RichText::new(props.name).size(13.0).color(Color32::from_rgb(148, 163, 184)));
                });
            } else if let Some(tex) = props.local_texture {
                // Live camera video texture rendering (16:9 responsive fit)
                let img_w = (props.width - 2.0).max(100.0);
                let img_h = (props.height - 2.0).max(80.0);
                ui.centered_and_justified(|ui| {
                    ui.image((tex.id(), Vec2::new(img_w, img_h)));
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.35);
                    ui.label(RichText::new("🎥 Video Stream").size(14.0).color(Color32::from_rgb(148, 163, 184)));
                    if props.is_local {
                        ui.label(RichText::new("(You)").size(11.0).color(Color32::from_rgb(100, 116, 139)));
                    }
                });
            }

            // Participant Name & Audio Pill in Bottom Left of Tile
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgba_premultiplied(11, 12, 14, 220))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut label = props.name.to_string();
                            if props.is_local {
                                label.push_str(" (You)");
                            }
                            ui.label(RichText::new(label).size(11.0).strong().color(Color32::from_rgb(248, 250, 252)));

                            if props.role == "host" {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(2, 132, 199))
                                    .stroke(Stroke::NONE)
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(4.0, 1.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new("HOST").size(9.0).strong().color(Color32::WHITE));
                                    });
                            }

                            if props.is_audio_muted {
                                ui.label(RichText::new("🔇").size(10.0));
                            } else if props.is_active_speaker {
                                ui.label(RichText::new("🔊").color(Color32::from_rgb(16, 185, 129)).size(10.0));
                            }

                            if props.is_hand_raised {
                                ui.label(RichText::new("✋").size(11.0));
                            }
                        });
                    });
            });
        });
}

fn render_reactions(app: &mut ConferApp, ui: &mut Ui, rect: Rect) {
    let now = std::time::Instant::now();
    app.active_reactions.retain(|r| now.duration_since(r.created_at).as_secs_f32() < 3.0);

    for reaction in &app.active_reactions {
        let elapsed = now.duration_since(reaction.created_at).as_secs_f32();
        let progress = (elapsed / 3.0).clamp(0.0, 1.0);
        let y_offset = progress * 160.0;
        let pos = Pos2::new(rect.center().x + reaction.x_offset, rect.bottom() - 120.0 - y_offset);

        ui.painter().text(
            pos,
            egui::Align2::CENTER_CENTER,
            &reaction.emoji,
            egui::FontId::proportional(32.0),
            Color32::WHITE,
        );
    }
}
