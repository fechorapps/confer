use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Stroke, Ui, Vec2};

pub fn render_waiting_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // Dark Obsidian canvas background
    ui.painter().rect_filled(full_rect, 0.0, Theme::CANVAS);

    // Top Header Bar
    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_1)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(0.0)
        .inner_margin(egui::Margin::symmetric(24.0, 14.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("CONFER")
                        .size(16.0)
                        .strong()
                        .color(Theme::PRIMARY_LIGHT),
                );
                ui.label(RichText::new("•").color(Theme::TEXT_MUTED));
                ui.label(
                    RichText::new("Waiting Room")
                        .size(14.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕ Leave").size(12.0).color(Color32::WHITE),
                            )
                            .fill(Theme::CRIMSON)
                            .rounding(Theme::RADIUS_SM),
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
            .fill(Theme::SURFACE_1)
            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
            .rounding(Theme::RADIUS_LG)
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
                ui.painter().circle_filled(center, 24.0, Theme::PRIMARY);
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
                        .color(Theme::TEXT_PRIMARY),
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
                        .color(Theme::TEXT_SECONDARY),
                );

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(16.0);

                // Participant Info Pill
                egui::Frame::group(ui.style())
                    .fill(Theme::SURFACE_2)
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                    .rounding(Theme::RADIUS_MD)
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
                                .fill(Theme::PRIMARY)
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
                                        .fill(Color32::from_rgb(45, 30, 10))
                                        .stroke(Stroke::new(1.0_f32, Theme::AMBER))
                                        .rounding(Theme::RADIUS_SM)
                                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("WAITING")
                                                    .size(10.0)
                                                    .strong()
                                                    .color(Theme::AMBER),
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
                            .color(Theme::TEXT_MUTED),
                    );

                    let mic_icon = if app.is_mic_muted {
                        "🔇 Mic Muted"
                    } else {
                        "🎙 Mic Ready"
                    };
                    let mic_col = if app.is_mic_muted {
                        Theme::CRIMSON
                    } else {
                        Theme::EMERALD
                    };
                    ui.label(RichText::new(mic_icon).size(11.0).color(mic_col));

                    ui.label(RichText::new("•").color(Theme::TEXT_MUTED));

                    let cam_icon = if app.is_camera_off {
                        "📷 Camera Off"
                    } else {
                        "🎥 Camera Ready"
                    };
                    let cam_col = if app.is_camera_off {
                        Theme::CRIMSON
                    } else {
                        Theme::EMERALD
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
                        .fill(Theme::CRIMSON)
                        .rounding(Theme::RADIUS_MD)
                        .min_size(Vec2::new(card_width - 24.0, 36.0)),
                    )
                    .clicked()
                {
                    app.leave_meeting();
                }
            });
    });
}
