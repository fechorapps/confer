mod dispatch;
mod header;
mod host_toolbar;
mod participant_row;
mod waiting_room;

use crate::app::{ConferApp, RosterTab};
use crate::ui::theme::Theme;
use egui::{ScrollArea, Stroke, Ui};

use dispatch::{dispatch_roster_actions, RosterActions};
use header::{render_header_bar, render_tab_selector};
use host_toolbar::render_host_toolbar;
use participant_row::{render_local_participant_row, render_participant_row};
use waiting_room::render_waiting_room_tab;

pub fn render_roster(app: &mut ConferApp, ui: &mut Ui) {
    let is_host = app.my_role == "host";
    let waiting_count = app.waiting_participants.len();
    let total_in_meeting = app.roster.len() + 1;

    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_1)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(Theme::RADIUS_MD)
        .inner_margin(12.0)
        .show(ui, |ui| {
            // Header Bar
            render_header_bar(app, ui);
            ui.add_space(8.0);

            // Segmented Tabs for Host: In-Meeting vs Waiting Room
            if is_host {
                render_tab_selector(app, ui, waiting_count, total_in_meeting);
                ui.add_space(8.0);
            }

            ui.separator();
            ui.add_space(6.0);

            let mut actions = RosterActions::default();

            if is_host && app.roster_tab == RosterTab::WaitingRoom {
                // --- WAITING ROOM TAB CONTENT ---
                let waiting_actions = render_waiting_room_tab(ui, app, waiting_count);
                actions.admit = waiting_actions.admit;
                actions.reject = waiting_actions.reject;
                actions.admit_all = waiting_actions.admit_all;
            } else {
                // --- IN-MEETING PARTICIPANTS TAB CONTENT ---
                // Host Toolbar: Mute All & Lock Controls
                if is_host && !app.roster.is_empty() {
                    actions.mute_all = render_host_toolbar(ui, app);
                    ui.add_space(6.0);
                }

                ScrollArea::vertical().show(ui, |ui| {
                    // Local participant row
                    render_local_participant_row(ui, app);
                    ui.add_space(6.0);

                    // Remote participants
                    for p in &app.roster {
                        let row_actions = render_participant_row(ui, p, is_host);
                        if row_actions.mute.is_some() {
                            actions.mute = row_actions.mute;
                        }
                        if row_actions.kick.is_some() {
                            actions.kick = row_actions.kick;
                        }
                        ui.add_space(4.0);
                    }
                });
            }

            // Dispatch pending host actions
            dispatch_roster_actions(app, actions);
        });
}
