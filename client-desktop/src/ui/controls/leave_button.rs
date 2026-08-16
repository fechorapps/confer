use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui, Vec2};

pub(super) fn render_leave_button(app: &mut ConferApp, ui: &mut Ui) {
    if ui
        .add_sized(
            Vec2::new(0.0, 32.0),
            egui::Button::new(
                RichText::new("✕ Leave")
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Theme::CRIMSON)
            .rounding(Theme::RADIUS_PILL),
        )
        .on_hover_text("Leave Meeting (Ctrl+W)")
        .clicked()
    {
        app.show_leave_confirmation = true;
    }
}
