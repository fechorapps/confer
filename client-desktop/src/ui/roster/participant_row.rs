use crate::app::ConferApp;
use crate::sdk::protocol::ParticipantState;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Stroke, Ui};
use uuid::Uuid;

/// Actions collected while rendering a single remote-participant row.
#[derive(Default)]
pub(super) struct ParticipantRowActions {
    pub mute: Option<Uuid>,
    pub kick: Option<(Uuid, String)>,
}

/// Local participant ("you") row: read-only.
pub(super) fn render_local_participant_row(ui: &mut Ui, app: &ConferApp) {
    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_2)
        .stroke(Stroke::NONE)
        .rounding(Theme::RADIUS_SM)
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let initial = app
                    .user_display_name
                    .chars()
                    .next()
                    .unwrap_or('U')
                    .to_uppercase()
                    .to_string();
                ui.label(
                    RichText::new(format!("({initial})"))
                        .size(11.0)
                        .color(Theme::PRIMARY_LIGHT),
                );
                ui.label(
                    RichText::new(&app.user_display_name)
                        .strong()
                        .size(12.0)
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("(You)")
                        .size(10.0)
                        .color(Theme::TEXT_SECONDARY),
                );

                if app.my_role == "host" {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("★ Host")
                                .size(10.0)
                                .strong()
                                .color(Theme::AMBER),
                        );
                    });
                }
            });
        });
}

/// Single remote-participant row: initial, name, host star, muted/hand-raised
/// icons, and (if host) Kick/Mute buttons.
///
/// Takes `p: &ParticipantState` (borrowed from `app.roster` by the caller's
/// loop) rather than `&ConferApp`, and returns the requested actions instead
/// of mutating `app` directly, so the caller can accumulate them and apply
/// them to `app` once the loop over `&app.roster` has ended.
pub(super) fn render_participant_row(
    ui: &mut Ui,
    p: &ParticipantState,
    is_host: bool,
) -> ParticipantRowActions {
    let mut actions = ParticipantRowActions::default();

    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_2)
        .stroke(Stroke::NONE)
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
                ui.label(
                    RichText::new(format!("({initial})"))
                        .size(11.0)
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.label(
                    RichText::new(&p.display_name)
                        .size(12.0)
                        .color(Theme::TEXT_PRIMARY),
                );

                if p.role == "host" {
                    ui.label(RichText::new("★").size(10.0).color(Theme::AMBER));
                }

                if p.is_audio_muted {
                    ui.label(RichText::new("🔇").size(10.0));
                }
                if p.is_hand_raised {
                    ui.label(RichText::new("✋").size(10.0));
                }

                // Host moderation actions
                if is_host {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Kick").size(9.5).color(Theme::CRIMSON_LIGHT),
                                )
                                .fill(Color32::from_rgb(45, 15, 20))
                                .rounding(Theme::RADIUS_SM),
                            )
                            .on_hover_text("Remove participant from call")
                            .clicked()
                        {
                            actions.kick = Some((p.user_id, p.display_name.clone()));
                        }

                        if !p.is_audio_muted
                            && ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Mute").size(9.5).color(Theme::TEXT_PRIMARY),
                                    )
                                    .fill(Theme::SURFACE_3)
                                    .rounding(Theme::RADIUS_SM),
                                )
                                .on_hover_text("Mute participant's microphone")
                                .clicked()
                        {
                            actions.mute = Some(p.user_id);
                        }
                    });
                }
            });
        });

    actions
}
