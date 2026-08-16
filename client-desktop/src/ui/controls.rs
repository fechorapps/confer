use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Stroke, Ui};

pub fn render_controls(app: &mut ConferApp, ui: &mut Ui) {
    let is_host = app.my_role == "host";

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(14.0);

        // Floating Frosted Glass Capsule Dock
        egui::Frame::group(ui.style())
            .fill(Theme::SURFACE_DOCK)
            .stroke(Stroke::new(1.0_f32, Theme::SURFACE_3))
            .rounding(Theme::RADIUS_PILL)
            .inner_margin(egui::Margin::symmetric(16.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // =========================================================
                    // CLUSTER 1: CORE HARDWARE MEDIA (MIC, CAM, SCREEN SHARE)
                    // =========================================================
                    let mic_bg = if app.is_push_to_talk_active {
                        Theme::EMERALD
                    } else if app.is_mic_muted {
                        Theme::CRIMSON
                    } else {
                        Theme::SURFACE_2
                    };

                    let level = if app.is_mic_muted { 0.0 } else { app.mic_test_level };
                    let mic_text = if app.is_push_to_talk_active {
                        "🎙 PTT Active".to_string()
                    } else if app.is_mic_muted {
                        "🔇 Unmute".to_string()
                    } else if level > 0.35 {
                        "🎙 Mute (●●●)".to_string()
                    } else if level > 0.12 {
                        "🎙 Mute (●●○)".to_string()
                    } else {
                        "🎙 Mute (●○○)".to_string()
                    };

                    let mic_hover = if app.is_mic_muted {
                        "Microphone Muted (Ctrl+D to unmute, or hold Spacebar for PTT)"
                    } else {
                        "Microphone Active (Ctrl+D to mute, audio level responding)"
                    };

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(mic_text)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(mic_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text(mic_hover)
                        .clicked()
                    {
                        app.toggle_mic();
                    }

                    let cam_bg = if app.is_camera_off {
                        Theme::CRIMSON
                    } else {
                        Theme::SURFACE_2
                    };
                    let cam_text = if app.is_camera_off {
                        "📷 Start Video"
                    } else {
                        "🎥 Stop Video"
                    };

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(cam_text)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(cam_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Toggle Video Camera (Ctrl+E)")
                        .clicked()
                    {
                        app.toggle_camera();
                    }

                    // Screen Share Control
                    if app.is_screen_sharing {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("⏹ Stop Share")
                                        .size(12.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Theme::CRIMSON)
                                .rounding(Theme::RADIUS_PILL),
                            )
                            .on_hover_text("Stop Screen Sharing (Ctrl+Shift+S)")
                            .clicked()
                        {
                            app.stop_screen_share();
                        }
                    } else if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("🖥 Share")
                                        .size(12.0)
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
                            RichText::new("🖥 Share").size(12.0).strong().color(Color32::WHITE),
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

                    // Cluster Divider
                    ui.add_space(4.0);
                    ui.painter().vline(
                        ui.cursor().min.x,
                        ui.available_rect_before_wrap().y_range(),
                        Stroke::new(1.0_f32, Theme::BORDER_SUBTLE),
                    );
                    ui.add_space(4.0);

                    // =========================================================
                    // CLUSTER 2: REAL-TIME COLLABORATION & AUDIENCE EXPRESSION
                    // =========================================================
                    let hand_bg = if app.is_hand_raised {
                        Theme::AMBER
                    } else {
                        Theme::SURFACE_2
                    };
                    let hand_text = if app.is_hand_raised { "✋ Lower" } else { "✋ Hand" };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(hand_text)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(hand_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Raise / Lower Hand")
                        .clicked()
                    {
                        app.toggle_hand_raise();
                    }

                    // Emoji Reactions Popover
                    ui.menu_button(
                        RichText::new("✨ React").size(12.0).strong().color(Color32::WHITE),
                        |ui| {
                            ui.horizontal(|ui| {
                                for emoji in ["👍", "❤️", "👏", "🎉", "🚀", "💡", "🔥"] {
                                    if ui
                                        .add(
                                            egui::Button::new(RichText::new(emoji).size(18.0))
                                                .fill(Theme::SURFACE_2)
                                                .rounding(Theme::RADIUS_SM),
                                        )
                                        .clicked()
                                    {
                                        app.send_reaction(emoji);
                                        ui.close_menu();
                                    }
                                }
                            });
                        },
                    );

                    let wb_bg = if app.is_whiteboard_active {
                        Theme::PRIMARY
                    } else {
                        Theme::SURFACE_2
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("🖌 Board")
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(wb_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Collaborative Whiteboard (Ctrl+Shift+W)")
                        .clicked()
                    {
                        app.toggle_whiteboard();
                    }

                    let polls_bg = if app.show_polls {
                        Theme::PRIMARY
                    } else {
                        Theme::SURFACE_2
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("📊 Polls")
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(polls_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Live Polls (Ctrl+Shift+P)")
                        .clicked()
                    {
                        app.toggle_polls();
                    }

                    let cc_bg = if app.is_captions_enabled {
                        Theme::PRIMARY
                    } else {
                        Theme::SURFACE_2
                    };
                    let cc_text = if app.is_captions_enabled { "💬 CC (On)" } else { "💬 CC" };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(cc_text)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(cc_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Live Speech-to-Text Subtitles (Ctrl+Shift+C)")
                        .clicked()
                    {
                        app.toggle_captions();
                    }

                    // Cluster Divider
                    ui.add_space(4.0);
                    ui.painter().vline(
                        ui.cursor().min.x,
                        ui.available_rect_before_wrap().y_range(),
                        Stroke::new(1.0_f32, Theme::BORDER_SUBTLE),
                    );
                    ui.add_space(4.0);

                    // =========================================================
                    // CLUSTER 3: SIDE PANELS, STUDIO FX & HOST GOVERNANCE
                    // =========================================================
                    let chat_btn_text = if app.unread_chat_count > 0 {
                        format!("💬 Chat ({})", app.unread_chat_count)
                    } else {
                        "💬 Chat".to_string()
                    };
                    let chat_bg = if app.show_chat {
                        Theme::PRIMARY
                    } else {
                        Theme::SURFACE_2
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(chat_btn_text)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(chat_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("In-call Chat (Ctrl+Shift+M)")
                        .clicked()
                    {
                        app.show_chat = !app.show_chat;
                        if app.show_chat {
                            app.show_roster = false;
                            app.show_polls = false;
                            app.unread_chat_count = 0;
                        }
                    }

                    // People Button with Amber Waiting Room Badge if guests are queued
                    let waiting_count = app.waiting_participants.len();
                    let roster_text = if waiting_count > 0 {
                        format!("👥 People ({} • {} ⏳)", app.roster.len() + 1, waiting_count)
                    } else {
                        format!("👥 People ({})", app.roster.len() + 1)
                    };
                    let roster_bg = if app.show_roster {
                        Theme::PRIMARY
                    } else if waiting_count > 0 {
                        Theme::AMBER
                    } else {
                        Theme::SURFACE_2
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(roster_text)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(roster_bg)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Participant Roster & Waiting Queue")
                        .clicked()
                    {
                        app.show_roster = !app.show_roster;
                        if app.show_roster {
                            app.show_chat = false;
                            app.show_polls = false;
                        }
                    }

                    // Studio FX & Video Tone Settings Menu (⚙)
                    ui.menu_button(
                        RichText::new("⚙ Studio FX").size(12.0).strong().color(Color32::WHITE),
                        |ui| {
                            ui.set_min_width(240.0);

                            // Audio & AI Denoise
                            ui.label(RichText::new("Audio FX & Denoise").size(11.0).strong().color(Theme::PRIMARY_LIGHT));
                            let denoise_label = if app.is_ai_denoise_enabled {
                                "✓ ⚡ RNNoise 48kHz (Active)"
                            } else {
                                "   ⚡ RNNoise 48kHz (Off)"
                            };
                            if ui.selectable_label(app.is_ai_denoise_enabled, denoise_label).clicked() {
                                app.toggle_ai_denoise();
                            }

                            ui.separator();

                            // Video Tone Filters
                            ui.label(RichText::new("Cinematic Tone Preset").size(11.0).strong().color(Theme::PRIMARY_LIGHT));
                            for filter in crate::media::filters::VideoFilter::all() {
                                let is_active = app.active_filter == *filter;
                                let label = format!("{}{}", if is_active { "✓ " } else { "   " }, filter.label());
                                if ui.selectable_label(is_active, label).clicked() {
                                    app.active_filter = *filter;
                                    ui.close_menu();
                                }
                            }

                            ui.separator();

                            // Virtual Backgrounds
                            ui.label(RichText::new("Background & Portrait Blur").size(11.0).strong().color(Theme::PRIMARY_LIGHT));
                            for mode in crate::media::VirtualBackgroundMode::all() {
                                let is_active = app.virtual_bg_mode == *mode;
                                let label = format!("{}{}", if is_active { "✓ " } else { "   " }, mode.label());
                                if ui.selectable_label(is_active, label).clicked() {
                                    app.set_virtual_bg_mode(mode.clone());
                                    ui.close_menu();
                                }
                            }
                            if ui.button("📁 Choose Custom Photo...").clicked() {
                                app.choose_custom_background();
                                ui.close_menu();
                            }

                            ui.separator();
                            ui.label(RichText::new("Keyboard Shortcuts:").size(10.0).color(Theme::TEXT_SECONDARY));
                            ui.label(
                                RichText::new("Space (Hold): PTT • Ctrl+D: Mic • Ctrl+E: Cam\nCtrl+Shift+S: Screen • Ctrl+Shift+W: Board\nCtrl+Shift+P: Polls • Ctrl+Shift+M: Chat")
                                    .size(9.5)
                                    .color(Theme::TEXT_MUTED),
                            );
                        },
                    );

                    // Host Security Policy Menu (Visible to Host)
                    if is_host {
                        ui.menu_button(
                            RichText::new("🛡 Security").size(12.0).strong().color(Theme::PRIMARY_LIGHT),
                            |ui| {
                                ui.set_min_width(220.0);
                                ui.label(RichText::new("Host Governance & DLP").size(11.0).strong().color(Theme::AMBER));
                                ui.separator();

                                let lock_label = if app.meeting_policy.is_locked {
                                    "✓ 🔒 Lock Meeting (Active)"
                                } else {
                                    "   🔒 Lock Meeting"
                                };
                                if ui.selectable_label(app.meeting_policy.is_locked, lock_label).clicked() {
                                    app.toggle_room_lock();
                                }

                                let wr_label = if app.meeting_policy.waiting_room_enabled {
                                    "✓ ⏳ Enable Waiting Room"
                                } else {
                                    "   ⏳ Enable Waiting Room"
                                };
                                if ui.selectable_label(app.meeting_policy.waiting_room_enabled, wr_label).clicked() {
                                    app.toggle_waiting_room(!app.meeting_policy.waiting_room_enabled);
                                }

                                let ss_label = if app.meeting_policy.allow_screen_share {
                                    "✓ 🖥 Allow Screen Share"
                                } else {
                                    "   🖥 Allow Screen Share"
                                };
                                if ui.selectable_label(app.meeting_policy.allow_screen_share, ss_label).clicked() {
                                    app.toggle_allow_screen_share();
                                }

                                let chat_label = if app.meeting_policy.allow_chat {
                                    "✓ 💬 Allow Chat"
                                } else {
                                    "   💬 Allow Chat"
                                };
                                if ui.selectable_label(app.meeting_policy.allow_chat, chat_label).clicked() {
                                    app.toggle_allow_chat();
                                }

                                let unmute_label = if app.meeting_policy.allow_unmute {
                                    "✓ 🎙 Allow Self Unmute"
                                } else {
                                    "   🎙 Allow Self Unmute"
                                };
                                if ui.selectable_label(app.meeting_policy.allow_unmute, unmute_label).clicked() {
                                    app.toggle_allow_unmute();
                                }

                                let wm_label = if app.meeting_policy.watermark_enabled {
                                    "✓ 🏷 Visual DLP Watermark"
                                } else {
                                    "   🏷 Visual DLP Watermark"
                                };
                                if ui.selectable_label(app.meeting_policy.watermark_enabled, wm_label).clicked() {
                                    app.toggle_watermark();
                                }
                            },
                        );
                    }

                    // Cluster Divider
                    ui.add_space(4.0);
                    ui.painter().vline(
                        ui.cursor().min.x,
                        ui.available_rect_before_wrap().y_range(),
                        Stroke::new(1.0_f32, Theme::BORDER_SUBTLE),
                    );
                    ui.add_space(4.0);

                    // =========================================================
                    // CLUSTER 4: HIGH-STAKES LEAVE ACTION (CONFIRMATION GUARD)
                    // =========================================================
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕ Leave")
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Theme::CRIMSON)
                            .rounding(Theme::RADIUS_PILL),
                        )
                        .on_hover_text("Leave Meeting")
                        .clicked()
                    {
                        app.show_leave_confirmation = true;
                    }
                });
            });
    });
}
