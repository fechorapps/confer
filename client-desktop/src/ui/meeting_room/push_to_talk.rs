use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

pub(super) fn render_push_to_talk_indicator(app: &ConferApp, ui: &mut Ui, full_rect: Rect) {
    if !app.is_push_to_talk_active {
        return;
    }

    let ptt_pos = Pos2::new(full_rect.center().x, full_rect.top() + 64.0);
    let ptt_rect = Rect::from_center_size(ptt_pos, Vec2::new(340.0, 36.0));

    ui.painter().rect_filled(
        ptt_rect,
        18.0,
        Color32::from_rgba_premultiplied(Theme::EMERALD.r(), Theme::EMERALD.g(), Theme::EMERALD.b(), 245),
    );
    ui.painter().rect_stroke(
        ptt_rect,
        18.0,
        Stroke::new(1.5_f32, crate::ui::theme::Theme::EMERALD_LIGHT),
    );

    ui.painter().text(
        ptt_pos,
        egui::Align2::CENTER_CENTER,
        "🎙 TRANSMITTING (Push-to-Talk: Spacebar)",
        egui::FontId::proportional(12.5),
        Theme::ON_ACCENT,
    );
}
