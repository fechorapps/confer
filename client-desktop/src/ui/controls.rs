use egui::{Color32, RichText, Stroke, Ui};
use crate::app::ConferApp;

pub fn render_controls(app: &mut ConferApp, ui: &mut Ui) {
    let is_host = app.my_role == "host";

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(14.0);

        // Floating Frosted Glass Pill Dock
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgba_premultiplied(18, 20, 23, 240))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
            .rounding(24.0)
            .inner_margin(egui::Margin::symmetric(18.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Mic Toggle (Respect host policy if non-host)
                    let mic_bg = if app.is_mic_muted { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                    let mic_text = if app.is_mic_muted { "🔇 Unmute" } else { "🎙 Mute" };
                    if ui.add(egui::Button::new(RichText::new(mic_text).size(12.0).color(Color32::WHITE)).fill(mic_bg).rounding(18.0)).clicked() {
                        app.toggle_mic();
                    }

                    // Camera Toggle
                    let cam_bg = if app.is_camera_off { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                    let cam_text = if app.is_camera_off { "📷 Start Video" } else { "🎥 Stop Video" };
                    if ui.add(egui::Button::new(RichText::new(cam_text).size(12.0).color(Color32::WHITE)).fill(cam_bg).rounding(18.0)).clicked() {
                        app.toggle_camera();
                    }

                    // Screen Share Control (Native Portal or DisplayList Dropdown)
                    if app.is_screen_sharing {
                        if ui.add(egui::Button::new(RichText::new("⏹ Stop Share").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(18.0)).clicked() {
                            app.stop_screen_share();
                        }
                    } else if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
                        if ui.add(egui::Button::new(RichText::new("🖥 Share Screen").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(26, 29, 33)).rounding(18.0)).clicked() {
                            app.start_native_screen_share();
                        }
                    } else {
                        ui.menu_button(RichText::new("🖥 Share").size(12.0).color(Color32::WHITE), |ui| {
                            ui.set_min_width(220.0);
                            ui.label(RichText::new("Select Display to Share").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                            ui.separator();
                            let displays = app.available_displays.clone();
                            for d in displays {
                                if ui.button(RichText::new(&d.label).size(11.0).color(Color32::WHITE)).clicked() {
                                    app.start_screen_share(d);
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    // AI Noise Suppression Toggle Button
                    let denoise_bg = if app.is_ai_denoise_enabled { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    let denoise_text = if app.is_ai_denoise_enabled { "⚡ AI Denoise" } else { "⚡ Denoise Off" };
                    if ui.add(egui::Button::new(RichText::new(denoise_text).size(12.0).color(Color32::WHITE)).fill(denoise_bg).rounding(18.0)).clicked() {
                        app.toggle_ai_denoise();
                    }

                    // Hand Raise
                    let hand_bg = if app.is_hand_raised { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(26, 29, 33) };
                    let hand_text = if app.is_hand_raised { "✋ Lower" } else { "✋ Hand" };
                    if ui.add(egui::Button::new(RichText::new(hand_text).size(12.0).color(Color32::WHITE)).fill(hand_bg).rounding(18.0)).clicked() {
                        app.toggle_hand_raise();
                    }

                    // Whiteboard Toggle
                    let wb_bg = if app.is_whiteboard_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new("🖌 Whiteboard").size(12.0).color(Color32::WHITE)).fill(wb_bg).rounding(18.0)).clicked() {
                        app.toggle_whiteboard();
                    }

                    // Polls Toggle
                    let polls_bg = if app.show_polls { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new("📊 Polls").size(12.0).color(Color32::WHITE)).fill(polls_bg).rounding(18.0)).clicked() {
                        app.toggle_polls();
                    }

                    // Live Captions / CC Subtitles Toggle
                    let cc_bg = if app.is_captions_enabled { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    let cc_text = if app.is_captions_enabled { "💬 CC (On)" } else { "💬 CC" };
                    if ui.add(egui::Button::new(RichText::new(cc_text).size(12.0).color(Color32::WHITE)).fill(cc_bg).rounding(18.0)).clicked() {
                        app.toggle_captions();
                    }


                    // Host Security Policy Menu Popup (🛡)
                    if is_host {
                        let sec_active = app.meeting_policy.is_locked || app.meeting_policy.waiting_room_enabled;
                        let sec_color = if sec_active { Color32::from_rgb(251, 191, 36) } else { Color32::WHITE };

                        ui.menu_button(RichText::new("🛡 Security").size(12.0).color(sec_color), |ui| {
                            ui.set_min_width(220.0);
                            ui.label(RichText::new("Host Security Policies").size(12.0).strong().color(Color32::from_rgb(56, 189, 248)));
                            ui.separator();

                            // Lock Meeting Toggle
                            let lock_label = if app.meeting_policy.is_locked { "✓ 🔒 Lock Meeting (Active)" } else { "   🔒 Lock Meeting" };
                            if ui.selectable_label(app.meeting_policy.is_locked, lock_label).clicked() {
                                app.toggle_room_lock();
                            }

                            // Waiting Room Toggle
                            let wr_label = if app.meeting_policy.waiting_room_enabled { "✓ ⏳ Enable Waiting Room" } else { "   ⏳ Enable Waiting Room" };
                            if ui.selectable_label(app.meeting_policy.waiting_room_enabled, wr_label).clicked() {
                                app.toggle_waiting_room(!app.meeting_policy.waiting_room_enabled);
                            }

                            ui.separator();
                            ui.label(RichText::new("Participant Permissions").size(10.0).strong().color(Color32::from_rgb(148, 163, 184)));

                            // Allow Screen Share
                            let ss_label = if app.meeting_policy.allow_screen_share { "✓ 🖥 Allow Screen Share" } else { "   🖥 Allow Screen Share" };
                            if ui.selectable_label(app.meeting_policy.allow_screen_share, ss_label).clicked() {
                                app.toggle_allow_screen_share();
                            }

                            // Allow Chat
                            let chat_label = if app.meeting_policy.allow_chat { "✓ 💬 Allow Chat" } else { "   💬 Allow Chat" };
                            if ui.selectable_label(app.meeting_policy.allow_chat, chat_label).clicked() {
                                app.toggle_allow_chat();
                            }

                            // Allow Unmute
                            let unmute_label = if app.meeting_policy.allow_unmute { "✓ 🎙 Allow Participants to Unmute" } else { "   🎙 Allow Participants to Unmute" };
                            if ui.selectable_label(app.meeting_policy.allow_unmute, unmute_label).clicked() {
                                app.toggle_allow_unmute();
                            }

                            ui.separator();
                            ui.label(RichText::new("Data Loss Prevention").size(10.0).strong().color(Color32::from_rgb(148, 163, 184)));

                            // Visual Watermark Toggle
                            let wm_label = if app.meeting_policy.watermark_enabled { "✓ 🏷 Visual Watermark" } else { "   🏷 Visual Watermark" };
                            if ui.selectable_label(app.meeting_policy.watermark_enabled, wm_label).clicked() {
                                app.toggle_watermark();
                            }
                        });
                    }

                    // Filters Dropdown
                    ui.menu_button(RichText::new("🎨 Filters").size(12.0).color(Color32::WHITE), |ui| {
                        ui.set_min_width(160.0);
                        ui.label(RichText::new("Video Filters (Meet)").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        ui.separator();
                        for filter in crate::media::filters::VideoFilter::all() {
                            let is_active = app.active_filter == *filter;
                            let label = format!("{}{}", if is_active { "✓ " } else { "   " }, filter.label());
                            if ui.selectable_label(is_active, label).clicked() {
                                app.active_filter = *filter;
                                ui.close_menu();
                            }
                        }
                    });

                    // Virtual Backgrounds & Blur Dropdown
                    ui.menu_button(RichText::new("🖼️ Background").size(12.0).color(Color32::WHITE), |ui| {
                        ui.set_min_width(200.0);
                        ui.label(RichText::new("Virtual Background & Blur").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        ui.separator();
                        for mode in crate::media::VirtualBackgroundMode::all() {
                            let is_active = app.virtual_bg_mode == *mode;
                            let label = format!("{}{}", if is_active { "✓ " } else { "   " }, mode.label());
                            if ui.selectable_label(is_active, label).clicked() {
                                app.set_virtual_bg_mode(mode.clone());
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("📁 Choose Custom Photo...").clicked() {
                            app.choose_custom_background();
                            ui.close_menu();
                        }
                    });

                    // Reactions Dropdown
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

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Red Leave Meeting Pill
                    if ui.add(egui::Button::new(RichText::new("✕ Leave").strong().size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(18.0)).clicked() {
                        app.leave_meeting();
                    }
                });
            });
    });
}
