use egui::{Color32, RichText, Stroke, Ui, Vec2};
use crate::app::ConferApp;

pub fn render_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let available_rect = ui.available_rect_before_wrap();

    // Background gradient canvas
    ui.painter().rect_filled(
        available_rect,
        0.0,
        Color32::from_rgb(11, 12, 14), // Obsidian
    );

    ui.vertical_centered(|ui| {
        ui.add_space(24.0);

        // Header Brand Bar
        ui.horizontal(|ui| {
            ui.add_space(32.0);
            ui.label(RichText::new("CONFER").size(22.0).strong().color(Color32::from_rgb(248, 250, 252)));
            ui.label(RichText::new("•").size(14.0).color(Color32::from_rgb(56, 189, 248)));
            ui.label(RichText::new("NATIVE CLIENT").size(11.0).color(Color32::from_rgb(148, 163, 184)));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(32.0);
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(18, 20, 23))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                    .rounding(20.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("●").size(9.0).color(Color32::from_rgb(16, 185, 129)));
                            ui.label(RichText::new(&app.server_url).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        });
                    });
            });
        });

        ui.add_space(20.0);

        // 2-Column Split Cockpit Layout
        ui.horizontal(|ui| {
            ui.add_space(32.0);
            let total_w = ui.available_width() - 32.0;
            let col_w = (total_w - 24.0) / 2.0;

            // Column 1: Live Hardware Video & Audio Stage
            ui.vertical(|ui| {
                ui.set_width(col_w);

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(18, 20, 23))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                    .rounding(12.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Device Pre-Check").size(14.0).strong().color(Color32::from_rgb(248, 250, 252)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let status_text = if app.is_camera_off { "Camera Disabled" } else { "✨ 720p HD • 30 FPS" };
                                let status_color = if app.is_camera_off { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(16, 185, 129) };
                                ui.label(RichText::new(status_text).size(11.0).color(status_color));
                            });
                        });
                        ui.add_space(10.0);

                        // Live Video Viewport (16:9 Aspect Ratio)
                        let viewport_h = col_w * 0.54;
                        egui::Frame::group(ui.style())
                            .fill(Color32::from_rgb(11, 12, 14))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(26, 29, 33)))
                            .rounding(8.0)
                            .inner_margin(0.0)
                            .show(ui, |ui| {
                                ui.set_width(col_w - 32.0);
                                ui.set_height(viewport_h);

                                if !app.is_camera_off {
                                    if let Some(tex) = &app.local_video_texture {
                                        ui.centered_and_justified(|ui| {
                                            ui.image((tex.id(), Vec2::new(col_w - 34.0, viewport_h - 2.0)));
                                        });
                                    }
                                } else {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(viewport_h * 0.35);
                                        let initial = app.user_display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                        egui::Frame::group(ui.style())
                                            .fill(Color32::from_rgb(38, 42, 48))
                                            .stroke(Stroke::NONE)
                                            .rounding(24.0)
                                            .inner_margin(12.0)
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(initial).size(24.0).strong().color(Color32::WHITE));
                                            });
                                        ui.add_space(8.0);
                                        ui.label(RichText::new("Camera is turned off").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                                    });
                                }
                            });

                        ui.add_space(14.0);

                        // Quick Toggle Buttons & Mic Visualizer
                        ui.horizontal(|ui| {
                            let cam_bg = if app.is_camera_off { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                            let cam_label = if app.is_camera_off { "📷 Camera Off" } else { "🎥 Camera On" };
                            if ui.add(egui::Button::new(RichText::new(cam_label).size(12.0).color(Color32::WHITE)).fill(cam_bg).rounding(6.0)).clicked() {
                                app.is_camera_off = !app.is_camera_off;
                            }

                            let mic_bg = if app.is_mic_muted { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                            let mic_label = if app.is_mic_muted { "🔇 Mic Muted" } else { "🎙 Mic Active" };
                            if ui.add(egui::Button::new(RichText::new(mic_label).size(12.0).color(Color32::WHITE)).fill(mic_bg).rounding(6.0)).clicked() {
                                app.is_mic_muted = !app.is_mic_muted;
                            }

                            let denoise_bg = if app.is_ai_denoise_enabled { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                            let denoise_label = if app.is_ai_denoise_enabled { "⚡ AI Denoise: ON" } else { "⚡ AI Denoise: OFF" };
                            if ui.add(egui::Button::new(RichText::new(denoise_label).size(12.0).color(Color32::WHITE)).fill(denoise_bg).rounding(6.0)).clicked() {
                                app.toggle_ai_denoise();
                            }
                        });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            let level = if app.is_mic_muted { 0.0 } else { app.mic_test_level };
                            ui.label(RichText::new("Mic Level:").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                            let bar_color = if level > 0.6 { Color32::from_rgb(244, 63, 94) } else { Color32::from_rgb(16, 185, 129) };
                            ui.add(egui::ProgressBar::new(level).desired_width(col_w - 120.0).fill(bar_color));
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Visual Filters Picker
                        ui.label(RichText::new("🎨 Visual Tone Filters").size(12.0).strong().color(Color32::from_rgb(248, 250, 252)));
                        ui.add_space(4.0);
                        egui::ScrollArea::horizontal().id_salt("filters_scroll").show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for filter in crate::media::filters::VideoFilter::all() {
                                    let is_active = app.active_filter == *filter;
                                    let bg = if is_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                    let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };
                                    if ui.add(egui::Button::new(RichText::new(filter.label()).size(11.0).color(text_color)).fill(bg).rounding(14.0)).clicked() {
                                        app.active_filter = *filter;
                                    }
                                }
                            });
                        });

                        ui.add_space(10.0);

                        // Virtual Backgrounds & Blur Picker
                        ui.label(RichText::new("🖼️ Background & Portrait Blur").size(12.0).strong().color(Color32::from_rgb(248, 250, 252)));
                        ui.add_space(4.0);
                        egui::ScrollArea::horizontal().id_salt("bg_scroll").show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for bg in crate::media::VirtualBackgroundMode::all() {
                                    let is_active = app.virtual_bg_mode == *bg;
                                    let btn_bg = if is_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                    let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };
                                    if ui.add(egui::Button::new(RichText::new(bg.label()).size(11.0).color(text_color)).fill(btn_bg).rounding(14.0)).clicked() {
                                        app.set_virtual_bg_mode(bg.clone());
                                    }
                                }

                                // Custom File Picker Button
                                let is_custom = matches!(app.virtual_bg_mode, crate::media::VirtualBackgroundMode::CustomImage(_));
                                let custom_bg = if is_custom { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                                if ui.add(egui::Button::new(RichText::new("📁 Custom Photo...").size(11.0).color(Color32::WHITE)).fill(custom_bg).rounding(14.0)).clicked() {
                                    app.choose_custom_background();
                                }
                            });
                        });
                    });
            });

            ui.add_space(24.0);

            // Column 2: Meeting Actions & User Persona
            ui.vertical(|ui| {
                ui.set_width(col_w);

                // User Profile Card
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(18, 20, 23))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                    .rounding(12.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Profile Persona").size(14.0).strong().color(Color32::from_rgb(248, 250, 252)));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            let is_host = app.user_email == "host@confer.local";
                            if ui.add(egui::Button::new(RichText::new("Host (Dev)").size(12.0)).fill(if is_host { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) }).rounding(6.0)).clicked() {
                                app.user_email = "host@confer.local".to_string();
                                app.user_display_name = "Host User (Dev)".to_string();
                            }
                            let is_alice = app.user_email == "participant1@confer.local";
                            if ui.add(egui::Button::new(RichText::new("Alice").size(12.0)).fill(if is_alice { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) }).rounding(6.0)).clicked() {
                                app.user_email = "participant1@confer.local".to_string();
                                app.user_display_name = "Alice (Dev)".to_string();
                            }
                            let is_bob = app.user_email == "participant2@confer.local";
                            if ui.add(egui::Button::new(RichText::new("Bob").size(12.0)).fill(if is_bob { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) }).rounding(6.0)).clicked() {
                                app.user_email = "participant2@confer.local".to_string();
                                app.user_display_name = "Bob (Dev)".to_string();
                            }
                        });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Display Name:").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                            ui.text_edit_singleline(&mut app.user_display_name);
                        });
                    });

                ui.add_space(16.0);

                // Meeting Actions Card
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(18, 20, 23))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                    .rounding(12.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Join or Create Meeting").size(14.0).strong().color(Color32::from_rgb(248, 250, 252)));
                        ui.add_space(12.0);

                        ui.label(RichText::new("Host New Meeting:").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut app.meeting_title_input);
                            if ui.add(egui::Button::new(RichText::new("✨ Start Meeting").strong().color(Color32::WHITE)).fill(Color32::from_rgb(2, 132, 199)).rounding(6.0)).clicked() {
                                app.trigger_create_meeting();
                            }
                        });

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label(RichText::new("Join by 6-Character Code:").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut app.join_code_input);
                            if ui.add(egui::Button::new(RichText::new("🚀 Join Meeting").strong().color(Color32::WHITE)).fill(Color32::from_rgb(16, 185, 129)).rounding(6.0)).clicked() {
                                app.trigger_join_meeting();
                            }
                        });

                        if let Some(err) = &app.error_message {
                            ui.add_space(12.0);
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(45, 15, 20))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(225, 29, 72)))
                                .rounding(6.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new(format!("⚠️ {err}")).size(11.0).color(Color32::from_rgb(254, 205, 211)));
                                });
                        }
                    });
            });
        });
    });
}
