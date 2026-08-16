use crate::app::ConferApp;
use egui::{Color32, Pos2, Rect, Ui};

pub(super) fn render_reactions(app: &mut ConferApp, ui: &mut Ui, full_rect: Rect) {
    let now = std::time::Instant::now();
    app.active_reactions
        .retain(|r| now.duration_since(r.created_at).as_secs_f32() < 3.0);

    for r in &app.active_reactions {
        let elapsed = now.duration_since(r.created_at).as_secs_f32();
        let y_offset = elapsed * 80.0;
        let x_pos = full_rect.left() + (full_rect.width() * r.x_offset);
        let y_pos = full_rect.bottom() - 100.0 - y_offset;

        let alpha_f = ((1.0 - (elapsed / 3.0)) * 255.0).clamp(0.0, 255.0);
        let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, alpha_f as u8);

        ui.painter().text(
            Pos2::new(x_pos, y_pos),
            egui::Align2::CENTER_CENTER,
            &r.emoji,
            egui::FontId::proportional(28.0),
            text_color,
        );
    }
}
