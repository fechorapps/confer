use crate::app::{ConferApp, RosterTab};
use crate::ui::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};
use uuid::Uuid;

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
            ui.horizontal(|ui| {
                ui.label(RichText::new("Participants").strong().size(14.0).color(Theme::TEXT_PRIMARY));
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
            ui.add_space(8.0);

            // Segmented Tabs for Host: In-Meeting vs Waiting Room
            if is_host {
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
                ui.add_space(8.0);
            }

            ui.separator();
            ui.add_space(6.0);

            let mut admit_action: Option<Uuid> = None;
            let mut reject_action: Option<Uuid> = None;
            let mut admit_all = false;
            let mut mute_action: Option<Uuid> = None;
            let mut mute_all = false;

            if is_host && app.roster_tab == RosterTab::WaitingRoom {
                // --- WAITING ROOM TAB CONTENT ---
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
                                admit_all = true;
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
                                                ui.label(
                                                    RichText::new(email)
                                                        .size(9.5)
                                                        .color(Theme::TEXT_SECONDARY),
                                                );
                                            }
                                        });

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            RichText::new("✕")
                                                                .size(10.0)
                                                                .color(Theme::CRIMSON_LIGHT),
                                                        )
                                                        .fill(Color32::from_rgb(45, 15, 20))
                                                        .rounding(Theme::RADIUS_SM),
                                                    )
                                                    .on_hover_text("Reject / Remove from waiting room")
                                                    .clicked()
                                                {
                                                    reject_action = Some(p.user_id);
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
                                                    admit_action = Some(p.user_id);
                                                }
                                            },
                                        );
                                    });
                                });
                            ui.add_space(4.0);
                        }
                    });
                }
            } else {
                // --- IN-MEETING PARTICIPANTS TAB CONTENT ---
                // Host Toolbar: Mute All & Lock Controls
                if is_host && !app.roster.is_empty() {
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
                                egui::Button::new(
                                    RichText::new(lock_label)
                                        .size(10.5)
                                        .strong()
                                        .color(if app.meeting_policy.is_locked {
                                            Theme::AMBER
                                        } else {
                                            Color32::WHITE
                                        }),
                                )
                                .fill(lock_bg)
                                .rounding(Theme::RADIUS_SM),
                            )
                            .clicked()
                        {
                            app.toggle_room_lock();
                        }
                    });
                    ui.add_space(6.0);
                }

                ScrollArea::vertical().show(ui, |ui| {
                    // Local participant row
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
                                ui.label(RichText::new("(You)").size(10.0).color(Theme::TEXT_SECONDARY));

                                if app.my_role == "host" {
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new("★ Host")
                                                    .size(10.0)
                                                    .strong()
                                                    .color(Theme::AMBER),
                                            );
                                        },
                                    );
                                }
                            });
                        });
                    ui.add_space(6.0);

                    // Remote participants
                    for p in &app.roster {
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
                                        ui.label(
                                            RichText::new("★")
                                                .size(10.0)
                                                .color(Theme::AMBER),
                                        );
                                    }

                                    if p.is_audio_muted {
                                        ui.label(RichText::new("🔇").size(10.0));
                                    }
                                    if p.is_hand_raised {
                                        ui.label(RichText::new("✋").size(10.0));
                                    }

                                    // Host moderation actions
                                    if is_host {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            RichText::new("Kick")
                                                                .size(9.5)
                                                                .color(Theme::CRIMSON_LIGHT),
                                                        )
                                                        .fill(Color32::from_rgb(45, 15, 20))
                                                        .rounding(Theme::RADIUS_SM),
                                                    )
                                                    .on_hover_text("Remove participant from call")
                                                    .clicked()
                                                {
                                                    app.kick_confirmation_target =
                                                        Some((p.user_id, p.display_name.clone()));
                                                }

                                                if !p.is_audio_muted
                                                    && ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("Mute")
                                                                    .size(9.5)
                                                                    .color(Theme::TEXT_PRIMARY),
                                                            )
                                                            .fill(Theme::SURFACE_3)
                                                            .rounding(Theme::RADIUS_SM),
                                                        )
                                                        .on_hover_text("Mute participant's microphone")
                                                        .clicked()
                                                {
                                                    mute_action = Some(p.user_id);
                                                }
                                            },
                                        );
                                    }
                                });
                            });
                        ui.add_space(4.0);
                    }
                });
            }

            // Dispatch pending host actions
            if let Some(id) = admit_action {
                app.admit_participant(id);
            }
            if let Some(id) = reject_action {
                app.reject_participant(id);
            }
            if admit_all {
                app.admit_all_waiting();
            }
            if let Some(id) = mute_action {
                app.host_mute_participant(id);
            }
            if mute_all {
                let to_mute: Vec<Uuid> = app
                    .roster
                    .iter()
                    .filter(|p| !p.is_audio_muted)
                    .map(|p| p.user_id)
                    .collect();
                for id in to_mute {
                    app.host_mute_participant(id);
                }
            }
        });
}
