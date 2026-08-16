use crate::app::ConferApp;
use crate::ui::components::Components;
use crate::ui::theme::Theme;
use egui::{Color32, Rect, RichText, Stroke, Ui, Vec2};

pub(super) fn render_leave_confirmation_modal(app: &mut ConferApp, ui: &mut Ui, full_rect: Rect) {
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
        let modal_rect =
            egui::Rect::from_center_size(overlay_rect.center(), Vec2::new(modal_w, modal_h));

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
                                        RichText::new("Stay in Queue")
                                            .size(12.0)
                                            .color(Theme::TEXT_PRIMARY),
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
