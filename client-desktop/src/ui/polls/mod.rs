use egui::{RichText, Stroke, Ui};

use crate::app::ConferApp;
use crate::ui::theme::Theme;

mod create_form;
mod list;
mod model;

#[allow(unused_imports)]
pub use model::compute_option_percentage;

pub fn render_polls(app: &mut ConferApp, ui: &mut Ui) {
    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_1)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            // --- Header ---
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📊 Live Polls & Voting")
                        .strong()
                        .size(14.0)
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("✕").size(12.0))
                                .fill(Theme::SURFACE_2)
                                .rounding(4.0),
                        )
                        .clicked()
                    {
                        app.show_polls = false;
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // --- Toggle between List View and Creation Dialog ---
            if app.poll_creating {
                create_form::render_poll_creation_form(app, ui);
            } else {
                list::render_polls_list(app, ui);
            }
        });
}
