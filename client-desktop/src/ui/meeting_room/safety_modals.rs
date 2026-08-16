use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, Rect, RichText, Stroke, Ui, Vec2};

pub(super) fn render_safety_modals(app: &mut ConferApp, ui: &mut Ui, full_rect: Rect) {
    // Leave Meeting Confirmation Modal
    if app.show_leave_confirmation {
        ui.painter().rect_filled(
            full_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        let modal_w = 400.0_f32;
        let modal_h = 180.0_f32;
        let center = full_rect.center();
        let modal_rect = Rect::from_center_size(center, Vec2::new(modal_w, modal_h));

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
            egui::Frame::group(ui.style())
                .fill(Theme::SURFACE_1)
                .stroke(Stroke::new(1.5_f32, Theme::CRIMSON))
                .rounding(16.0)
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("⚠️ Leave Meeting?")
                                .size(17.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Are you sure you want to disconnect from this conference room?",
                            )
                            .size(12.5)
                            .color(Theme::TEXT_SECONDARY),
                        );
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            ui.add_space(32.0);
                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Cancel (Esc)")
                                            .size(12.5)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Theme::SURFACE_3)
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                app.show_leave_confirmation = false;
                            }

                            ui.add_space(16.0);

                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Leave Call (Enter)")
                                            .size(12.5)
                                            .strong()
                                            .color(Theme::ON_ACCENT),
                                    )
                                    .fill(Theme::CRIMSON)
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                app.show_leave_confirmation = false;
                                app.leave_meeting();
                            }
                        });
                    });
                });
        });
    }

    // Host Kick Participant Confirmation Modal
    if let Some((target_id, target_name)) = app.kick_confirmation_target.clone() {
        ui.painter().rect_filled(
            full_rect,
            0.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        let modal_w = 420.0_f32;
        let modal_h = 190.0_f32;
        let center = full_rect.center();
        let modal_rect = Rect::from_center_size(center, Vec2::new(modal_w, modal_h));

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(modal_rect), |ui| {
            egui::Frame::group(ui.style())
                .fill(Theme::SURFACE_1)
                .stroke(Stroke::new(1.5_f32, Theme::CRIMSON))
                .rounding(16.0)
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Remove Participant?")
                                .size(17.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "Are you sure you want to remove '{target_name}' from this call?"
                            ))
                            .size(12.5)
                            .color(Theme::TEXT_SECONDARY),
                        );
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            ui.add_space(36.0);
                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Cancel (Esc)")
                                            .size(12.5)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Theme::SURFACE_3)
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Escape))
                            {
                                app.kick_confirmation_target = None;
                            }

                            ui.add_space(16.0);

                            if ui
                                .add_sized(
                                    Vec2::new(140.0, 36.0),
                                    egui::Button::new(
                                        RichText::new("Remove (Enter)")
                                            .size(12.5)
                                            .strong()
                                            .color(Theme::ON_ACCENT),
                                    )
                                    .fill(Theme::CRIMSON)
                                    .rounding(8.0),
                                )
                                .clicked()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                app.host_kick_participant(target_id);
                                app.kick_confirmation_target = None;
                            }
                        });
                    });
                });
        });
    }
}
