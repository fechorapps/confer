use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui};

/// Host toolbar: "Mute All" + "Lock Room/Unlock" buttons.
///
/// Returns `true` if "Mute All" was clicked (collected `mute_all` flag).
/// The room-lock toggle is applied directly since it has no deferred
/// borrow-checker concerns.
pub(super) fn render_host_toolbar(ui: &mut Ui, app: &mut ConferApp) -> bool {
    let mut mute_all = false;

    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    RichText::new("🔇 Mute All")
                        .size(10.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Theme::SURFACE_3)
                .rounding(Theme::RADIUS_SM),
            )
            .on_hover_text("Mute all remote attendees")
            .clicked()
        {
            mute_all = true;
        }

        let lock_label = if app.meeting_policy.is_locked {
            "🔓 Unlock"
        } else {
            "🔒 Lock Room"
        };
        let lock_bg = if app.meeting_policy.is_locked {
            Color32::from_rgb(45, 30, 10)
        } else {
            Theme::SURFACE_3
        };
        if ui
            .add(
                egui::Button::new(RichText::new(lock_label).size(10.5).strong().color(
                    if app.meeting_policy.is_locked {
                        Theme::AMBER
                    } else {
                        Color32::WHITE
                    },
                ))
                .fill(lock_bg)
                .rounding(Theme::RADIUS_SM),
            )
            .clicked()
        {
            app.toggle_room_lock();
        }
    });

    mute_all
}
