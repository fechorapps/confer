use egui::{Color32, RichText, Stroke, Ui};
use crate::app::ConferApp;

pub fn render_controls(app: &mut ConferApp, ui: &mut Ui) {
    let is_host = app.my_role == "host";

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(14.0);

        // Floating Frosted Glass Pill Dock
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgba_premultiplied(18, 20, 23, 245))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
            .rounding(24.0)
            .inner_margin(egui::Margin::symmetric(16.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // 1. Core Media Controls (Mic & Video)
                    let mic_bg = if app.is_mic_muted { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                    let mic_text = if app.is_push_to_talk_active {
                        "🎙 Transmitting"
                    } else if app.is_mic_muted {
                        "🔇 Unmute"
                    } else {
                        "🎙 Mute"
                    };
                    let mic_btn = egui::Button::new(RichText::new(mic_text).size(12.0).color(Color32::WHITE))
                        .fill(mic_bg)
                        .rounding(18.0);
                    if ui.add(mic_btn).on_hover_text("Toggle Microphone (Ctrl+D / Spacebar Hold for PTT)").clicked() {
                        app.toggle_mic();
                    }

                    let cam_bg = if app.is_camera_off { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                    let cam_text = if app.is_camera_off { "📷 Start Video" } else { "🎥 Stop Video" };
                    let cam_btn = egui::Button::new(RichText::new(cam_text).size(12.0).color(Color32::WHITE))
                        .fill(cam_bg)
                        .rounding(18.0);
                    if ui.add(cam_btn).on_hover_text("Toggle Video Camera (Ctrl+E)").clicked() {
                        app.toggle_camera();
                    }

                    // Screen Share Control
                    if app.is_screen_sharing {
                        if ui.add(egui::Button::new(RichText::new("⏹ Stop Share").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(18.0)).on_hover_text("Stop Screen Sharing (Ctrl+Shift+S)").clicked() {
                            app.stop_screen_share();
                        }
                    } else if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
                        if ui.add(egui::Button::new(RichText::new("🖥 Share").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(26, 29, 33)).rounding(18.0)).on_hover_text("Share Screen (Ctrl+Shift+S)").clicked() {
                            app.start_native_screen_share();
                        }
                    } else {
                        let mut selected_display: Option<usize> = None;
                        ui.menu_button(RichText::new("🖥 Share").size(12.0).color(Color32::WHITE), |ui| {
                            ui.set_min_width(220.0);
                            ui.label(RichText::new("Select Display to Share").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                            ui.separator();
                            for (idx, d) in app.available_displays.iter().enumerate() {
                                if ui.button(RichText::new(&d.label).size(11.0).color(Color32::WHITE)).clicked() {
                                    selected_display = Some(idx);
                                    ui.close_menu();
                                }
                            }
                        });
                        if let Some(idx) = selected_display {
                            if let Some(d) = app.available_displays.get(idx).cloned() {
                                app.start_screen_share(d);
                            }
                        }
                    }

                    // Hand Raise
                    let hand_bg = if app.is_hand_raised { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(26, 29, 33) };
                    let hand_text = if app.is_hand_raised { "✋ Lower" } else { "✋ Hand" };
                    if ui.add(egui::Button::new(RichText::new(hand_text).size(12.0).color(Color32::WHITE)).fill(hand_bg).rounding(18.0)).clicked() {
                        app.toggle_hand_raise();
                    }

                    // Emoji Reactions Popover
                    ui.menu_button(RichText::new("✨ React").size(12.0).color(Color32::WHITE), |ui| {
                        ui.horizontal(|ui| {
                            for emoji in ["👍", "❤️", "👏", "🎉", "🚀", "💡", "🔥"] {
                                if ui.add(egui::Button::new(RichText::new(emoji).size(18.0)).fill(Color32::from_rgb(26, 29, 33)).rounding(6.0)).clicked() {
                                    app.send_reaction(emoji);
                                    ui.close_menu();
                                }
                            }
                        });
                    });

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // 2. Collaboration Suite (Whiteboard, Polls, Captions)
                    let wb_bg = if app.is_whiteboard_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new("🖌 Whiteboard").size(12.0).color(Color32::WHITE)).fill(wb_bg).rounding(18.0)).on_hover_text("Collaborative Whiteboard (Ctrl+Shift+W)").clicked() {
                        app.toggle_whiteboard();
                    }

                    let polls_bg = if app.show_polls { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new("📊 Polls").size(12.0).color(Color32::WHITE)).fill(polls_bg).rounding(18.0)).on_hover_text("Live Polls (Ctrl+Shift+P)").clicked() {
                        app.toggle_polls();
                    }

                    let cc_bg = if app.is_captions_enabled { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    let cc_text = if app.is_captions_enabled { "💬 CC (On)" } else { "💬 CC" };
                    if ui.add(egui::Button::new(RichText::new(cc_text).size(12.0).color(Color32::WHITE)).fill(cc_bg).rounding(18.0)).on_hover_text("Live Speech-to-Text Subtitles (Ctrl+Shift+C)").clicked() {
                        app.toggle_captions();
                    }

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(2.0);

                    // 3. Side Panel Toggles (Chat & People)
                    let chat_btn_text = if app.unread_chat_count > 0 {
                        format!("💬 Chat ({})", app.unread_chat_count)
                    } else {
                        "💬 Chat".to_string()
                    };
                    let chat_bg = if app.show_chat { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new(chat_btn_text).size(12.0).color(Color32::WHITE)).fill(chat_bg).rounding(18.0)).on_hover_text("In-call Chat (Ctrl+Shift+M)").clicked() {
                        app.show_chat = !app.show_chat;
                        if app.show_chat {
                            app.show_roster = false;
                            app.show_polls = false;
                            app.unread_chat_count = 0;
                        }
                    }

                    let roster_bg = if app.show_roster { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new(format!("👥 People ({})", app.roster.len() + 1)).size(12.0).color(Color32::WHITE)).fill(roster_bg).rounding(18.0)).on_hover_text("Participants Roster & Waiting Room").clicked() {
                        app.show_roster = !app.show_roster;
                        if app.show_roster {
                            app.show_chat = false;
                            app.show_polls = false;
                        }
                    }

                    // 4. Unified Settings & Controls Menu (⚙)
                    ui.menu_button(RichText::new("⚙ Settings").size(12.0).color(Color32::WHITE), |ui| {
                        ui.set_min_width(260.0);

                        // AI Noise Suppression Section
                        ui.label(RichText::new("Audio & AI").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        let denoise_label = if app.is_ai_denoise_enabled { "✓ ⚡ AI Noise Suppression (Active)" } else { "   ⚡ AI Noise Suppression (Off)" };
                        if ui.selectable_label(app.is_ai_denoise_enabled, denoise_label).clicked() {
                            app.toggle_ai_denoise();
                        }

                        ui.separator();

                        // Video Tone Filters Section
                        ui.label(RichText::new("Video Tone Filters").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        for filter in crate::media::filters::VideoFilter::all() {
                            let is_active = app.active_filter == *filter;
                            let label = format!("{}{}", if is_active { "✓ " } else { "   " }, filter.label());
                            if ui.selectable_label(is_active, label).clicked() {
                                app.active_filter = *filter;
                                ui.close_menu();
                            }
                        }

                        ui.separator();

                        // Virtual Backgrounds Section
                        ui.label(RichText::new("Virtual Background & Blur").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
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

                        // Host Security Policies Section (if host)
                        if is_host {
                            ui.separator();
                            ui.label(RichText::new("Host Security Policies").size(11.0).strong().color(Color32::from_rgb(251, 191, 36)));

                            let lock_label = if app.meeting_policy.is_locked { "✓ 🔒 Lock Meeting (Active)" } else { "   🔒 Lock Meeting" };
                            if ui.selectable_label(app.meeting_policy.is_locked, lock_label).clicked() {
                                app.toggle_room_lock();
                            }

                            let wr_label = if app.meeting_policy.waiting_room_enabled { "✓ ⏳ Enable Waiting Room" } else { "   ⏳ Enable Waiting Room" };
                            if ui.selectable_label(app.meeting_policy.waiting_room_enabled, wr_label).clicked() {
                                app.toggle_waiting_room(!app.meeting_policy.waiting_room_enabled);
                            }

                            let ss_label = if app.meeting_policy.allow_screen_share { "✓ 🖥 Allow Screen Share" } else { "   🖥 Allow Screen Share" };
                            if ui.selectable_label(app.meeting_policy.allow_screen_share, ss_label).clicked() {
                                app.toggle_allow_screen_share();
                            }

                            let chat_label = if app.meeting_policy.allow_chat { "✓ 💬 Allow Chat" } else { "   💬 Allow Chat" };
                            if ui.selectable_label(app.meeting_policy.allow_chat, chat_label).clicked() {
                                app.toggle_allow_chat();
                            }

                            let unmute_label = if app.meeting_policy.allow_unmute { "✓ 🎙 Allow Self Unmute" } else { "   🎙 Allow Self Unmute" };
                            if ui.selectable_label(app.meeting_policy.allow_unmute, unmute_label).clicked() {
                                app.toggle_allow_unmute();
                            }

                            let wm_label = if app.meeting_policy.watermark_enabled { "✓ 🏷 Visual DLP Watermark" } else { "   🏷 Visual DLP Watermark" };
                            if ui.selectable_label(app.meeting_policy.watermark_enabled, wm_label).clicked() {
                                app.toggle_watermark();
                            }
                        }

                        ui.separator();

                        // Diagnostics HUD
                        let diag_label = if app.show_diagnostics { "✓ 📊 Health HUD (Active)" } else { "   📊 Health HUD (RTT/FPS/RAM)" };
                        if ui.selectable_label(app.show_diagnostics, diag_label).clicked() {
                            app.show_diagnostics = !app.show_diagnostics;
                        }

                        ui.separator();
                        ui.label(RichText::new("Keyboard Shortcuts:").size(10.0).color(Color32::from_rgb(148, 163, 184)));
                        ui.label(RichText::new("Space (Hold): Push-to-Talk\nCtrl+D: Mic • Ctrl+E: Camera\nCtrl+Shift+S: Share Screen\nCtrl+Shift+W: Whiteboard\nCtrl+Shift+P: Polls • Esc: Close").size(9.5).color(Color32::from_rgb(100, 116, 139)));
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // 5. Red Leave Meeting Button (Opens Confirmation Modal for Safety)
                    if ui.add(egui::Button::new(RichText::new("✕ Leave").strong().size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(18.0)).on_hover_text("Leave Conference").clicked() {
                        app.show_leave_confirmation = true;
                    }
                });
            });
    });
}
