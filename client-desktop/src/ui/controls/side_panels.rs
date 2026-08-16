use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui, Vec2};

pub(super) fn render_side_panel_buttons(app: &mut ConferApp, ui: &mut Ui) {
    let chat_btn_text = if app.unread_chat_count > 0 {
        format!("💬 Chat ({})", app.unread_chat_count)
    } else {
        "💬 Chat".to_string()
    };
    let chat_bg = if app.show_chat {
        Theme::PRIMARY
    } else {
        Theme::SURFACE_2
    };
    if ui
        .add_sized(
            Vec2::new(0.0, 32.0),
            egui::Button::new(
                RichText::new(chat_btn_text)
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(chat_bg)
            .rounding(Theme::RADIUS_PILL),
        )
        .on_hover_text("In-call Chat (Ctrl+Shift+M)")
        .clicked()
    {
        app.show_chat = !app.show_chat;
        if app.show_chat {
            app.show_roster = false;
            app.show_polls = false;
            app.unread_chat_count = 0;
        }
    }

    // People Button with Amber Waiting Room Badge if guests are queued
    let waiting_count = app.waiting_participants.len();
    let roster_text = if waiting_count > 0 {
        format!(
            "👥 People ({} • {} ⏳)",
            app.roster.len() + 1,
            waiting_count
        )
    } else {
        format!("👥 People ({})", app.roster.len() + 1)
    };
    let roster_bg = if app.show_roster {
        Theme::PRIMARY
    } else if waiting_count > 0 {
        Theme::AMBER
    } else {
        Theme::SURFACE_2
    };
    if ui
        .add_sized(
            Vec2::new(0.0, 32.0),
            egui::Button::new(
                RichText::new(roster_text)
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(roster_bg)
            .rounding(Theme::RADIUS_PILL),
        )
        .on_hover_text("Participant Roster & Waiting Queue")
        .clicked()
    {
        app.show_roster = !app.show_roster;
        if app.show_roster {
            app.show_chat = false;
            app.show_polls = false;
        }
    }
}
