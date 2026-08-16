use crate::app::ConferApp;
use crate::sdk::protocol::WaitingParticipantDto;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};
use uuid::Uuid;

/// Actions collected while rendering the Waiting Room tab.
#[derive(Default)]
pub(super) struct WaitingRoomActions {
    pub admit: Option<Uuid>,
    pub reject: Option<Uuid>,
    pub admit_all: bool,
}

/// Action requested from a single waiting-participant row.
pub(super) enum WaitingRowAction {
    Admit,
    Reject,
}

/// Waiting-room tab: header + "Admit All" + empty state + scrollable list.
pub(super) fn render_waiting_room_tab(
    ui: &mut Ui,
    app: &ConferApp,
    waiting_count: usize,
) -> WaitingRoomActions {
    let mut actions = WaitingRoomActions::default();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Waiting in Lobby ({waiting_count})"))
                .size(12.0)
                .strong()
                .color(Theme::AMBER),
        );
        if waiting_count > 0 {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("✓ Admit All")
                                .size(10.5)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(Theme::EMERALD)
                        .rounding(Theme::RADIUS_SM),
                    )
                    .clicked()
                {
                    actions.admit_all = true;
                }
            });
        }
    });
    ui.add_space(8.0);

    if waiting_count == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.label(
                RichText::new("No participants currently waiting.")
                    .size(12.0)
                    .color(Theme::TEXT_MUTED),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "New joiners will appear here when the Waiting Room policy is enabled.",
                )
                .size(10.0)
                .color(Theme::TEXT_DIM),
            );
        });
    } else {
        ScrollArea::vertical().show(ui, |ui| {
            for p in &app.waiting_participants {
                match render_waiting_participant_row(ui, p) {
                    Some(WaitingRowAction::Admit) => actions.admit = Some(p.user_id),
                    Some(WaitingRowAction::Reject) => actions.reject = Some(p.user_id),
                    None => {}
                }
            }
        });
    }

    actions
}

/// Single waiting-participant card row.
fn render_waiting_participant_row(
    ui: &mut Ui,
    p: &WaitingParticipantDto,
) -> Option<WaitingRowAction> {
    let mut action = None;

    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_2)
        .stroke(Stroke::new(1.0_f32, Theme::SURFACE_3))
        .rounding(Theme::RADIUS_SM)
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let initial = p
                    .display_name
                    .chars()
                    .next()
                    .unwrap_or('U')
                    .to_uppercase()
                    .to_string();
                egui::Frame::group(ui.style())
                    .fill(Theme::SURFACE_3)
                    .stroke(Stroke::NONE)
                    .rounding(12.0)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(initial)
                                .size(10.0)
                                .strong()
                                .color(Theme::PRIMARY_LIGHT),
                        );
                    });
                ui.add_space(4.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&p.display_name)
                            .size(11.5)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    if let Some(email) = &p.email {
                        ui.label(RichText::new(email).size(9.5).color(Theme::TEXT_SECONDARY));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕").size(10.0).color(Theme::CRIMSON_LIGHT),
                            )
                            .fill(Color32::from_rgb(45, 15, 20))
                            .rounding(Theme::RADIUS_SM),
                        )
                        .on_hover_text("Reject / Remove from waiting room")
                        .clicked()
                    {
                        action = Some(WaitingRowAction::Reject);
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✓ Admit")
                                    .size(10.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Theme::EMERALD)
                            .rounding(Theme::RADIUS_SM),
                        )
                        .on_hover_text("Admit to meeting room")
                        .clicked()
                    {
                        action = Some(WaitingRowAction::Admit);
                    }
                });
            });
        });
    ui.add_space(4.0);

    action
}
