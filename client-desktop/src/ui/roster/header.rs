use crate::app::{ConferApp, RosterTab};
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui};

/// Header bar: title + close (✕) button.
pub(super) fn render_header_bar(app: &mut ConferApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Participants")
                .strong()
                .size(14.0)
                .color(Theme::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("✕").size(12.0).color(Theme::TEXT_SECONDARY))
                        .fill(Theme::SURFACE_2)
                        .rounding(Theme::RADIUS_SM),
                )
                .clicked()
            {
                app.show_roster = false;
            }
        });
    });
}

/// Segmented Tabs for Host: In-Meeting vs Waiting Room.
pub(super) fn render_tab_selector(
    app: &mut ConferApp,
    ui: &mut Ui,
    waiting_count: usize,
    total_in_meeting: usize,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        let in_meeting_active = app.roster_tab == RosterTab::InMeeting;
        let in_meeting_bg = if in_meeting_active {
            Theme::PRIMARY
        } else {
            Theme::SURFACE_2
        };
        if ui
            .add(
                egui::Button::new(
                    RichText::new(format!("In Meeting ({total_in_meeting})"))
                        .size(11.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(in_meeting_bg)
                .rounding(Theme::RADIUS_SM),
            )
            .clicked()
        {
            app.roster_tab = RosterTab::InMeeting;
        }

        let waiting_active = app.roster_tab == RosterTab::WaitingRoom;
        let waiting_bg = if waiting_active {
            Theme::PRIMARY
        } else if waiting_count > 0 {
            Color32::from_rgb(45, 30, 10)
        } else {
            Theme::SURFACE_2
        };
        let waiting_text_color = if waiting_count > 0 && !waiting_active {
            Theme::AMBER
        } else {
            Color32::WHITE
        };

        let waiting_label = if waiting_count > 0 {
            format!("⏳ Waiting ({waiting_count})")
        } else {
            "Waiting Room (0)".to_string()
        };

        if ui
            .add(
                egui::Button::new(
                    RichText::new(waiting_label)
                        .size(11.0)
                        .strong()
                        .color(waiting_text_color),
                )
                .fill(waiting_bg)
                .rounding(Theme::RADIUS_SM),
            )
            .clicked()
        {
            app.roster_tab = RosterTab::WaitingRoom;
        }
    });
}
