use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{RichText, Stroke, Ui};

pub(super) fn render_header_bar(app: &mut ConferApp, ui: &mut Ui) {
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
                        ui.label(RichText::new("⚡").size(13.0).color(Theme::ON_ACCENT));
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
                                RichText::new("✕ Leave Queue")
                                    .size(12.0)
                                    .color(Theme::ON_ACCENT),
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
                                        RichText::new("PIN:").size(10.5).color(Theme::TEXT_MUTED),
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
}
