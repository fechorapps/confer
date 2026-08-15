use crate::app::ConferApp;
use egui::{Color32, RichText, Stroke, Ui, Vec2};

pub fn render_waiting_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // Dark Obsidian canvas background
    ui.painter()
        .rect_filled(full_rect, 0.0, Color32::from_rgb(11, 12, 14));

    // Top Header Bar
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 23))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(0.0)
        .inner_margin(egui::Margin::symmetric(24.0, 14.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("CONFER")
                        .size(16.0)
                        .strong()
                        .color(Color32::from_rgb(56, 189, 248)),
                );
                ui.label(RichText::new("•").color(Color32::from_rgb(71, 85, 105)));
                ui.label(
                    RichText::new("Waiting Room")
                        .size(14.0)
                        .strong()
                        .color(Color32::from_rgb(248, 250, 252)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕ Leave Meeting")
                                    .size(12.0)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(225, 29, 72))
                            .rounding(6.0),
                        )
                        .clicked()
                    {
                        app.leave_meeting();
                    }
                });
            });
        });

    // Centered Waiting Lobby Card
    let available_size = ui.available_size();
    let card_width = 460.0_f32.min(available_size.x - 32.0);

    ui.vertical_centered(|ui| {
        ui.add_space((available_size.y * 0.12).max(20.0));

        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(18, 20, 23))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
            .rounding(16.0)
            .inner_margin(28.0)
            .show(ui, |ui| {
                ui.set_width(card_width);

                // Animated Radar / Pulse Ring
                let time_sec = ui.input(|i| i.time);
                let pulse_scale = 1.0 + (time_sec * 3.0).sin().abs() as f32 * 0.2;
                let ring_radius = 28.0 * pulse_scale;

                let (rect, _response) =
                    ui.allocate_exact_size(Vec2::new(card_width, 80.0), egui::Sense::hover());
                let center = rect.center();

                // Outer pulsing glow circle
                ui.painter().circle_stroke(
                    center,
                    ring_radius + 8.0,
                    Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(56, 189, 248, 80)),
                );
                // Inner solid circle
                ui.painter()
                    .circle_filled(center, 24.0, Color32::from_rgb(2, 132, 199));
                // Hourglass / Shield icon inside circle
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "⏳",
                    egui::FontId::proportional(20.0),
                    Color32::WHITE,
                );

                ui.add_space(12.0);

                // Main Title
                let title = if app.room_title.is_empty() {
                    "Meeting Room"
                } else {
                    app.room_title.as_str()
                };
                ui.label(
                    RichText::new(title)
                        .size(20.0)
                        .strong()
                        .color(Color32::from_rgb(248, 250, 252)),
                );

                ui.add_space(6.0);

                // Status message
                let waiting_msg = app
                    .waiting_room_message
                    .as_deref()
                    .unwrap_or("Please wait, the meeting host will let you in soon.");
                ui.label(
                    RichText::new(waiting_msg)
                        .size(13.0)
                        .color(Color32::from_rgb(148, 163, 184)),
                );

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(16.0);

                // Participant Info Pill
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(26, 29, 33))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
                    .rounding(10.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let initial = app
                                .user_display_name
                                .chars()
                                .next()
                                .unwrap_or('U')
                                .to_uppercase()
                                .to_string();
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(2, 132, 199))
                                .stroke(Stroke::NONE)
                                .rounding(16.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(initial)
                                            .size(14.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                });

                            ui.add_space(6.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&app.user_display_name)
                                        .size(13.0)
                                        .strong()
                                        .color(Color32::from_rgb(248, 250, 252)),
                                );
                                if !app.user_email.is_empty() {
                                    ui.label(
                                        RichText::new(&app.user_email)
                                            .size(11.0)
                                            .color(Color32::from_rgb(148, 163, 184)),
                                    );
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    egui::Frame::group(ui.style())
                                        .fill(Color32::from_rgb(45, 30, 10))
                                        .stroke(Stroke::new(
                                            1.0_f32,
                                            Color32::from_rgb(245, 158, 11),
                                        ))
                                        .rounding(6.0)
                                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("WAITING")
                                                    .size(10.0)
                                                    .strong()
                                                    .color(Color32::from_rgb(251, 191, 36)),
                                            );
                                        });
                                },
                            );
                        });
                    });

                ui.add_space(16.0);

                // Quick Device Check in Lobby
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Pre-join check:")
                            .size(11.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );

                    let mic_icon = if app.is_mic_muted {
                        "🔇 Mic Muted"
                    } else {
                        "🎙 Mic Ready"
                    };
                    let mic_col = if app.is_mic_muted {
                        Color32::from_rgb(244, 63, 94)
                    } else {
                        Color32::from_rgb(16, 185, 129)
                    };
                    ui.label(RichText::new(mic_icon).size(11.0).color(mic_col));

                    ui.label(RichText::new("•").color(Color32::from_rgb(71, 85, 105)));

                    let cam_icon = if app.is_camera_off {
                        "📷 Camera Off"
                    } else {
                        "🎥 Camera Ready"
                    };
                    let cam_col = if app.is_camera_off {
                        Color32::from_rgb(244, 63, 94)
                    } else {
                        Color32::from_rgb(16, 185, 129)
                    };
                    ui.label(RichText::new(cam_icon).size(11.0).color(cam_col));
                });

                ui.add_space(20.0);

                // Leave Meeting Button
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("✕ Leave Waiting Room")
                                .size(13.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(225, 29, 72))
                        .rounding(8.0)
                        .min_size(Vec2::new(card_width - 24.0, 36.0)),
                    )
                    .clicked()
                {
                    app.leave_meeting();
                }
            });
    });
}
