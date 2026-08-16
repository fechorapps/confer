use crate::app::ConferApp;
use crate::ui::components::Components;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, RichText, Stroke, Ui, Vec2};

pub fn render_waiting_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // Dark Obsidian canvas background
    ui.painter().rect_filled(full_rect, 0.0, Theme::CANVAS);

    // 1. Top Global Header Bar with Unified Brand Mark & Leave Protection
    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_1)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(0.0)
        .inner_margin(egui::Margin::symmetric(24.0, 12.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Brand Logomark
                egui::Frame::group(ui.style())
                    .fill(Theme::PRIMARY)
                    .stroke(Stroke::new(1.0_f32, Theme::PRIMARY_LIGHT))
                    .rounding(Theme::RADIUS_MD)
                    .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("⚡").size(13.0).color(Color32::WHITE));
                    });
                ui.add_space(8.0);
                ui.label(
                    RichText::new("CONFER")
                        .size(16.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("STUDIO")
                        .size(10.0)
                        .strong()
                        .color(Theme::PRIMARY_LIGHT),
                );
                ui.label(RichText::new("•").size(10.0).color(Theme::TEXT_MUTED));
                ui.label(
                    RichText::new("Waiting Lounge")
                        .size(13.0)
                        .color(Theme::TEXT_SECONDARY),
                );

                // Right Hub: Room PIN & Confirmed Leave
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕ Leave Queue").size(12.0).color(Color32::WHITE),
                            )
                            .fill(Theme::CRIMSON)
                            .rounding(Theme::RADIUS_SM),
                        )
                        .clicked()
                    {
                        app.show_leave_confirmation = true;
                    }

                    if let Some(code) = &app.current_join_code {
                        ui.add_space(12.0);
                        egui::Frame::group(ui.style())
                            .fill(Theme::SURFACE_2)
                            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                            .rounding(Theme::RADIUS_PILL)
                            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("PIN:")
                                            .size(10.5)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                    ui.label(
                                        RichText::new(code)
                                            .size(11.0)
                                            .strong()
                                            .font(egui::FontId::monospace(11.5))
                                            .color(Theme::PRIMARY_LIGHT),
                                    );
                                });
                            });
                    }
                });
            });
        });

    // 2. Centered Pre-Admission Lounge Card
    let available_size = ui.available_size();
    let card_width = 480.0_f32.min(available_size.x - 32.0);

    ui.vertical_centered(|ui| {
        ui.add_space((available_size.y * 0.08).max(16.0));

        Theme::card_frame(ui.style()).show(ui, |ui| {
            ui.set_width(card_width);

            // Smooth Harmonic Sonar / Pulse Animation (No mathematical cusp)
            let time_sec = ui.input(|i| i.time);
            let pulse_harmonic = 1.0 + (time_sec * 2.0).sin() as f32 * 0.08;
            let ring_radius = 26.0 * pulse_harmonic;

            let (rect, _response) =
                ui.allocate_exact_size(Vec2::new(card_width, 70.0), egui::Sense::hover());
            let center = rect.center();

            // Outer smooth glow ring
            ui.painter().circle_stroke(
                center,
                ring_radius + 6.0,
                Stroke::new(
                    1.5_f32,
                    Color32::from_rgba_premultiplied(Theme::PRIMARY_LIGHT.r(), Theme::PRIMARY_LIGHT.g(), Theme::PRIMARY_LIGHT.b(), 70),
                ),
            );
            // Inner solid primary circle
            ui.painter().circle_filled(center, 22.0, Theme::PRIMARY);

            // Crisp Vector Shield & Clock Glyph
            let p = ui.painter();
            let icon_col = Color32::WHITE;
            p.circle_stroke(center, 10.0, Stroke::new(1.5_f32, icon_col));
            p.line_segment([center, Pos2::new(center.x, center.y - 6.0)], Stroke::new(1.5_f32, icon_col));
            p.line_segment([center, Pos2::new(center.x + 4.0, center.y)], Stroke::new(1.5_f32, icon_col));

            ui.add_space(8.0);

            // Room Title & Off-Air Security Reassurance
            let title = if app.room_title.is_empty() {
                "Meeting Room"
            } else {
                app.room_title.as_str()
            };
            ui.label(
                RichText::new(title)
                    .size(19.0)
                    .strong()
                    .color(Theme::TEXT_PRIMARY),
            );

            ui.add_space(4.0);

            // Status message from host
            let waiting_msg = app
                .waiting_room_message
                .as_deref()
                .unwrap_or("Please wait, the meeting host will let you in soon.");
            ui.label(
                RichText::new(waiting_msg)
                    .size(12.5)
                    .color(Theme::TEXT_SECONDARY),
            );

            ui.add_space(10.0);

            // Reassurance Off-Air Banner
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(15, 23, 42)) // Deep Slate
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 41, 59)))
                .rounding(Theme::RADIUS_PILL)
                .inner_margin(egui::Margin::symmetric(12.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔒").size(10.0).color(Theme::PRIMARY_LIGHT));
                        ui.label(
                            RichText::new("Off Air — You are not visible or audible until admitted")
                                .size(11.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                    });
                });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(12.0);

            // Participant Profile Identity Strip
            let user_initials = Components::extract_initials(&app.user_display_name);
            let persona_color = if app.user_email == "host@confer.local" {
                Theme::PRIMARY
            } else if app.user_email == "participant1@confer.local" {
                Color32::from_rgb(139, 92, 246)
            } else {
                Color32::from_rgb(13, 148, 136)
            };

            egui::Frame::group(ui.style())
                .fill(Theme::SURFACE_2)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Theme::RADIUS_MD)
                .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        Components::avatar_badge(ui, &user_initials, 28.0, 12.0, persona_color);
                        ui.add_space(6.0);

                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&app.user_display_name)
                                    .size(13.0)
                                    .strong()
                                    .color(Theme::TEXT_PRIMARY),
                            );
                            if !app.user_email.is_empty() {
                                ui.label(
                                    RichText::new(&app.user_email)
                                        .size(11.0)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            }
                        });

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(69, 26, 3)) // Deep Amber Fill
                                    .stroke(Stroke::new(1.0_f32, Theme::AMBER))
                                    .rounding(Theme::RADIUS_PILL)
                                    .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("● IN QUEUE")
                                                .size(11.0)
                                                .strong()
                                                .color(Theme::AMBER),
                                        );
                                    });
                            },
                        );
                    });
                });

            ui.add_space(14.0);

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
                            RichText::new(cam_text).size(11.0).strong().color(Color32::WHITE),
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
                            RichText::new(mic_text).size(11.0).strong().color(Color32::WHITE),
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
                if ui
                    .add_sized(
                        Vec2::new(btn_w, 32.0),
                        egui::Button::new(
                            RichText::new(denoise_text)
                                .size(11.0)
                                .strong()
                                .color(Color32::WHITE),
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

            ui.add_space(10.0);

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
        });
    });

    // 3. Leave Queue Confirmation Modal Dialog
    if app.show_leave_confirmation {
        let overlay_rect = full_rect;
        ui.painter().rect_filled(
            overlay_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        let modal_w = 380.0_f32;
        let modal_h = 180.0_f32;
        let modal_rect = egui::Rect::from_center_size(
            overlay_rect.center(),
            Vec2::new(modal_w, modal_h),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
            egui::Frame::group(ui.style())
                .fill(Theme::SURFACE_1)
                .stroke(Stroke::new(1.5_f32, Theme::PRIMARY_LIGHT))
                .rounding(Theme::RADIUS_LG)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Leave Waiting Room?")
                                .size(16.0)
                                .strong()
                                .color(Theme::TEXT_PRIMARY),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Leaving now will drop your place in the host admission queue.",
                            )
                            .size(12.0)
                            .color(Theme::TEXT_SECONDARY),
                        );
                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            let btn_w = 150.0_f32;
                            if ui
                                .add_sized(
                                    Vec2::new(btn_w, 34.0),
                                    egui::Button::new(
                                        RichText::new("Stay in Queue").size(12.0).color(Theme::TEXT_PRIMARY),
                                    )
                                    .fill(Theme::SURFACE_2)
                                    .rounding(Theme::RADIUS_SM),
                                )
                                .clicked()
                            {
                                app.show_leave_confirmation = false;
                            }

                            ui.add_space(12.0);

                            if ui
                                .add_sized(
                                    Vec2::new(btn_w, 34.0),
                                    Components::destructive_button("Leave Queue", 12.0),
                                )
                                .clicked()
                            {
                                app.show_leave_confirmation = false;
                                app.leave_meeting();
                            }
                        });
                    });
                });
        });
    }
}
