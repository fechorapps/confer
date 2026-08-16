use crate::app::ConferApp;
use crate::ui::components::Components;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, RichText, Stroke, Ui, Vec2};

pub(super) fn render_pulse_icon(ui: &mut Ui, card_width: f32) {
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
    let icon_col = Theme::ON_ACCENT;
    p.circle_stroke(center, 10.0, Stroke::new(1.5_f32, icon_col));
    p.line_segment([center, Pos2::new(center.x, center.y - 6.0)], Stroke::new(1.5_f32, icon_col));
    p.line_segment([center, Pos2::new(center.x + 4.0, center.y)], Stroke::new(1.5_f32, icon_col));
}

pub(super) fn render_room_status(app: &ConferApp, ui: &mut Ui) {
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
}

pub(super) fn render_offair_banner(ui: &mut Ui) {
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
}

pub(super) fn render_identity_strip(app: &ConferApp, ui: &mut Ui) {
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
}
