use egui::{Color32, RichText, Stroke, Ui, Vec2};
use crate::app::ConferApp;
use crate::media::filters::VideoFilter;
use crate::media::VirtualBackgroundMode;

pub fn render_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let available_rect = ui.available_rect_before_wrap();

    // 1. Deep Obsidian canvas with subtle luxury gradient effect
    ui.painter().rect_filled(
        available_rect,
        0.0,
        Color32::from_rgb(11, 12, 14), // Obsidian
    );

    // Decorative ambient background glows (deep indigo / electric cyan halo)
    ui.painter().circle_filled(
        available_rect.left_top() + egui::vec2(160.0, 100.0),
        220.0,
        Color32::from_rgba_premultiplied(2, 132, 199, 12),
    );
    ui.painter().circle_filled(
        available_rect.right_bottom() - egui::vec2(180.0, 120.0),
        260.0,
        Color32::from_rgba_premultiplied(16, 185, 129, 8),
    );

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // ==========================================
            // TOP GLOBAL NAVIGATION & SYSTEM STATUS BAR
            // ==========================================
            ui.horizontal(|ui| {
                ui.add_space(36.0);

                // Brand Mark & Logomark
                ui.horizontal(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(2, 132, 199))
                        .stroke(Stroke::NONE)
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡").size(14.0).color(Color32::WHITE));
                        });
                    ui.add_space(6.0);
                    ui.label(RichText::new("CONFER").size(20.0).strong().color(Color32::from_rgb(248, 250, 252)));
                    ui.label(RichText::new("PRO STUDIO").size(10.0).strong().color(Color32::from_rgb(56, 189, 248)));
                });

                // Center / Right Status Hub
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(36.0);

                    // User Identity Badge / Switcher
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(18, 20, 23))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                        .rounding(20.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let initial = app.user_display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(2, 132, 199))
                                    .stroke(Stroke::NONE)
                                    .rounding(10.0)
                                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(initial).size(10.0).strong().color(Color32::WHITE));
                                    });
                                ui.label(RichText::new(&app.user_display_name).size(12.0).strong().color(Color32::from_rgb(248, 250, 252)));
                                ui.label(RichText::new("•").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                ui.label(RichText::new(&app.user_email).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                            });
                        });

                    ui.add_space(8.0);

                    // Server Health Status Pill
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(18, 20, 23))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                        .rounding(20.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("●").size(8.0).color(Color32::from_rgb(16, 185, 129)));
                                ui.label(RichText::new("Core SFU Online").size(11.0).strong().color(Color32::from_rgb(203, 213, 225)));
                                ui.label(RichText::new("•").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                ui.label(RichText::new("<40MB Native").size(10.0).color(Color32::from_rgb(148, 163, 184)));
                            });
                        });
                });
            });

            ui.add_space(20.0);

            // ==========================================
            // MAIN 2-COLUMN STUDIO COCKPIT LAYOUT
            // ==========================================
            ui.horizontal(|ui| {
                ui.add_space(36.0);
                let total_w = ui.available_width() - 36.0;
                let col_w = (total_w - 28.0) / 2.0;

                // ------------------------------------------
                // COLUMN 1: LIVE STUDIO VIEWFINDER & AUDIO STAGE
                // ------------------------------------------
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    // Main Viewfinder Card
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(18, 20, 24))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                        .rounding(16.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            // Stage Header
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Studio Viewfinder").size(15.0).strong().color(Color32::from_rgb(248, 250, 252)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let status_text = if app.is_camera_off {
                                        "● Camera Inactive"
                                    } else {
                                        "● 720p HD • 30 FPS"
                                    };
                                    let status_color = if app.is_camera_off {
                                        Color32::from_rgb(148, 163, 184)
                                    } else {
                                        Color32::from_rgb(16, 185, 129)
                                    };
                                    egui::Frame::group(ui.style())
                                        .fill(Color32::from_rgb(11, 12, 14))
                                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                                        .rounding(12.0)
                                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new(status_text).size(11.0).strong().color(status_color));
                                        });
                                });
                            });

                            ui.add_space(12.0);

                            // 16:9 Video Canvas Surface
                            let viewport_w = (col_w - 40.0).max(100.0);
                            let viewport_h = (viewport_w * 0.5625).max(80.0);

                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(11, 12, 14))
                                .stroke(Stroke::new(1.5_f32, Color32::from_rgb(26, 29, 33)))
                                .rounding(12.0)
                                .inner_margin(0.0)
                                .show(ui, |ui| {
                                    ui.set_width(viewport_w);
                                    ui.set_height(viewport_h);

                                    if !app.is_camera_off {
                                        if let Some(tex) = &app.local_video_texture {
                                            ui.centered_and_justified(|ui| {
                                                ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(viewport_w, viewport_h)).rounding(12.0));
                                            });
                                        }
                                    } else {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(viewport_h * 0.3);
                                            let initial = app.user_display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                            egui::Frame::group(ui.style())
                                                .fill(Color32::from_rgb(26, 29, 33))
                                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 58)))
                                                .rounding(36.0)
                                                .inner_margin(18.0)
                                                .show(ui, |ui| {
                                                    ui.label(RichText::new(initial).size(32.0).strong().color(Color32::WHITE));
                                                });
                                            ui.add_space(10.0);
                                            ui.label(RichText::new("Camera is paused").size(13.0).color(Color32::from_rgb(148, 163, 184)));
                                        });
                                    }
                                });

                            ui.add_space(14.0);

                            // Quick Floating Hardware Controls
                            ui.horizontal(|ui| {
                                // Camera Toggle
                                let cam_bg = if app.is_camera_off { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                                let cam_text = if app.is_camera_off { "📷 Camera Off" } else { "🎥 Camera On" };
                                if ui.add_sized(
                                    Vec2::new(120.0, 34.0),
                                    egui::Button::new(RichText::new(cam_text).size(12.0).strong().color(Color32::WHITE)).fill(cam_bg).rounding(8.0),
                                ).on_hover_text("Toggle Camera (Ctrl+E)").clicked() {
                                    app.toggle_camera();
                                }

                                ui.add_space(4.0);

                                // Mic Toggle
                                let mic_bg = if app.is_mic_muted { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                                let mic_text = if app.is_mic_muted { "🔇 Mic Muted" } else { "🎙 Mic Active" };
                                if ui.add_sized(
                                    Vec2::new(120.0, 34.0),
                                    egui::Button::new(RichText::new(mic_text).size(12.0).strong().color(Color32::WHITE)).fill(mic_bg).rounding(8.0),
                                ).on_hover_text("Toggle Microphone (Ctrl+D)").clicked() {
                                    app.toggle_mic();
                                }

                                ui.add_space(4.0);

                                // AI Denoise Toggle
                                let denoise_bg = if app.is_ai_denoise_enabled { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                let denoise_text = if app.is_ai_denoise_enabled { "⚡ AI Denoise: ON" } else { "⚡ AI Denoise: OFF" };
                                if ui.add_sized(
                                    Vec2::new(135.0, 34.0),
                                    egui::Button::new(RichText::new(denoise_text).size(12.0).strong().color(Color32::WHITE)).fill(denoise_bg).rounding(8.0),
                                ).on_hover_text("RNNoise 48kHz Neural Noise Suppression").clicked() {
                                    app.toggle_ai_denoise();
                                }
                            });

                            ui.add_space(14.0);

                            // Studio Audio VU Visualizer
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Mic Energy:").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                let level = if app.is_mic_muted { 0.0 } else { app.mic_test_level };

                                // Segmented LED style meter
                                let total_segments = 24;
                                let active_segments = ((level * total_segments as f32).round() as usize).min(total_segments);
                                let segment_w = ((viewport_w - 90.0) / total_segments as f32) - 2.0;

                                for i in 0..total_segments {
                                    let is_lit = i < active_segments;
                                    let seg_color = if !is_lit {
                                        Color32::from_rgb(26, 29, 33)
                                    } else if i > 20 {
                                        Color32::from_rgb(244, 63, 94) // Red Peak
                                    } else if i > 15 {
                                        Color32::from_rgb(245, 158, 11) // Amber High
                                    } else {
                                        Color32::from_rgb(16, 185, 129) // Emerald Good
                                    };
                                    let seg_rect = ui.allocate_exact_size(Vec2::new(segment_w.max(2.0), 10.0), egui::Sense::hover()).0;
                                    ui.painter().rect_filled(seg_rect, 2.0, seg_color);
                                }
                            });

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(10.0);

                            // Visual Tone Filters
                            ui.label(RichText::new("🎨 Cinematic Color Tone").size(12.0).strong().color(Color32::from_rgb(248, 250, 252)));
                            ui.add_space(6.0);
                            egui::ScrollArea::horizontal().id_salt("tone_filters_scroll").show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for filter in VideoFilter::all() {
                                        let is_active = app.active_filter == *filter;
                                        let bg = if is_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                        let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };
                                        if ui.add(egui::Button::new(RichText::new(filter.label()).size(11.0).strong().color(text_color)).fill(bg).rounding(16.0)).clicked() {
                                            app.active_filter = *filter;
                                        }
                                    }
                                });
                            });

                            ui.add_space(12.0);

                            // Background & Portrait Blur Selector
                            ui.label(RichText::new("🖼️ Studio Background & Blur").size(12.0).strong().color(Color32::from_rgb(248, 250, 252)));
                            ui.add_space(6.0);
                            egui::ScrollArea::horizontal().id_salt("bg_modes_scroll").show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for bg in VirtualBackgroundMode::all() {
                                        let is_active = app.virtual_bg_mode == *bg;
                                        let btn_bg = if is_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                        let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };
                                        if ui.add(egui::Button::new(RichText::new(bg.label()).size(11.0).strong().color(text_color)).fill(btn_bg).rounding(16.0)).clicked() {
                                            app.set_virtual_bg_mode(bg.clone());
                                        }
                                    }

                                    let is_custom = matches!(app.virtual_bg_mode, VirtualBackgroundMode::CustomImage(_));
                                    let custom_bg = if is_custom { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                    if ui.add(egui::Button::new(RichText::new("📁 Custom Photo...").size(11.0).strong().color(Color32::WHITE)).fill(custom_bg).rounding(16.0)).clicked() {
                                        app.choose_custom_background();
                                    }
                                });
                            });
                        });
                });

                ui.add_space(28.0);

                // ------------------------------------------
                // COLUMN 2: MEETING LAUNCHER & PROFILE HUB
                // ------------------------------------------
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    // Card 1: Persona & Identity Hub
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(18, 20, 24))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                        .rounding(16.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Participant Profile").size(15.0).strong().color(Color32::from_rgb(248, 250, 252)));
                            ui.add_space(6.0);
                            ui.label(RichText::new("Select a pre-configured account or customize your identity:").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(12.0);

                            // Quick Persona Switcher Chips
                            ui.horizontal(|ui| {
                                let is_host = app.user_email == "host@confer.local";
                                if ui.add_sized(
                                    Vec2::new(100.0, 32.0),
                                    egui::Button::new(RichText::new("★ Host (Dev)").size(11.5).strong().color(if is_host { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) }))
                                        .fill(if is_host { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) })
                                        .rounding(8.0),
                                ).clicked() {
                                    app.user_email = "host@confer.local".to_string();
                                    app.user_display_name = "Host User (Dev)".to_string();
                                }

                                let is_alice = app.user_email == "participant1@confer.local";
                                if ui.add_sized(
                                    Vec2::new(100.0, 32.0),
                                    egui::Button::new(RichText::new("Alice (Guest)").size(11.5).strong().color(if is_alice { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) }))
                                        .fill(if is_alice { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) })
                                        .rounding(8.0),
                                ).clicked() {
                                    app.user_email = "participant1@confer.local".to_string();
                                    app.user_display_name = "Alice (Dev)".to_string();
                                }

                                let is_bob = app.user_email == "participant2@confer.local";
                                if ui.add_sized(
                                    Vec2::new(100.0, 32.0),
                                    egui::Button::new(RichText::new("Bob (Guest)").size(11.5).strong().color(if is_bob { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) }))
                                        .fill(if is_bob { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) })
                                        .rounding(8.0),
                                ).clicked() {
                                    app.user_email = "participant2@confer.local".to_string();
                                    app.user_display_name = "Bob (Dev)".to_string();
                                }
                            });

                            ui.add_space(14.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Your Name:").size(12.0).strong().color(Color32::from_rgb(203, 213, 225)));
                                ui.add_sized(
                                    Vec2::new(col_w - 140.0, 30.0),
                                    egui::TextEdit::singleline(&mut app.user_display_name)
                                        .hint_text("Enter your full name...")
                                        .margin(egui::Margin::symmetric(8.0, 4.0)),
                                );
                            });
                        });

                    ui.add_space(16.0);

                    // Card 2: Host New Instant Meeting
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(18, 20, 24))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(2, 132, 199)))
                        .rounding(16.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("✨ Start New Meeting").size(16.0).strong().color(Color32::WHITE));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    egui::Frame::group(ui.style())
                                        .fill(Color32::from_rgb(12, 74, 110))
                                        .stroke(Stroke::NONE)
                                        .rounding(10.0)
                                        .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("HOST").size(10.0).strong().color(Color32::from_rgb(56, 189, 248)));
                                        });
                                });
                            });

                            ui.add_space(6.0);
                            ui.label(RichText::new("Create an instant encrypted conference room and invite team members.").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(12.0);

                            // Meeting Title Input
                            ui.label(RichText::new("Meeting Topic:").size(12.0).strong().color(Color32::from_rgb(203, 213, 225)));
                            ui.add_space(4.0);
                            ui.add_sized(
                                Vec2::new(col_w - 40.0, 32.0),
                                egui::TextEdit::singleline(&mut app.meeting_title_input)
                                    .hint_text("e.g. Sprint Planning, Architecture Review...")
                                    .margin(egui::Margin::symmetric(10.0, 6.0)),
                            );

                            ui.add_space(14.0);

                            // Big Vibrant Start Meeting Action Button
                            if ui.add_sized(
                                Vec2::new(col_w - 40.0, 40.0),
                                egui::Button::new(RichText::new("🚀 Start Meeting Now").size(13.5).strong().color(Color32::WHITE))
                                    .fill(Color32::from_rgb(2, 132, 199))
                                    .rounding(10.0),
                            ).clicked() {
                                app.trigger_create_meeting();
                            }
                        });

                    ui.add_space(16.0);

                    // Card 3: Join Meeting by 6-Character Code
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(18, 20, 24))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                        .rounding(16.0)
                        .inner_margin(20.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("🔗 Join with Code").size(15.0).strong().color(Color32::from_rgb(248, 250, 252)));
                            ui.add_space(6.0);
                            ui.label(RichText::new("Enter the 6-character room code provided by the meeting host:").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.add_space(12.0);

                            // Room Code Input with Uppercase Normalization
                            ui.horizontal(|ui| {
                                let edit_res = ui.add_sized(
                                    Vec2::new(col_w - 180.0, 36.0),
                                    egui::TextEdit::singleline(&mut app.join_code_input)
                                        .hint_text("CODE (e.g. ABC123)")
                                        .font(egui::FontId::monospace(14.0))
                                        .margin(egui::Margin::symmetric(10.0, 6.0)),
                                );
                                app.join_code_input = app.join_code_input.to_uppercase();

                                let can_join = !app.join_code_input.trim().is_empty();
                                let join_bg = if can_join { Color32::from_rgb(16, 185, 129) } else { Color32::from_rgb(26, 29, 33) };
                                let join_text_color = if can_join { Color32::WHITE } else { Color32::from_rgb(100, 116, 139) };

                                if (ui.add_sized(
                                    Vec2::new(120.0, 36.0),
                                    egui::Button::new(RichText::new("Join Room").size(13.0).strong().color(join_text_color))
                                        .fill(join_bg)
                                        .rounding(8.0),
                                ).clicked() || (edit_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && can_join {
                                    app.trigger_join_meeting();
                                }
                            });

                            // Display connection error banner if present
                            if let Some(err) = &app.error_message {
                                ui.add_space(14.0);
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(45, 15, 20))
                                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(225, 29, 72)))
                                    .rounding(8.0)
                                    .inner_margin(10.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("⚠️").size(14.0));
                                            ui.label(RichText::new(err).size(12.0).color(Color32::from_rgb(254, 205, 211)));
                                        });
                                    });
                            }
                        });
                });
            });

            ui.add_space(32.0);
        });
    });
}
