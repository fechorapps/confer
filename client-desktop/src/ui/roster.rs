use egui::{Color32, RichText, ScrollArea, Stroke, Ui};
use uuid::Uuid;
use crate::app::{ConferApp, RosterTab};

pub fn render_roster(app: &mut ConferApp, ui: &mut Ui) {
    let is_host = app.my_role == "host";
    let waiting_count = app.waiting_participants.len();
    let total_in_meeting = app.roster.len() + 1;

    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 23))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("Participants").strong().size(14.0).color(Color32::from_rgb(248, 250, 252)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("✕").size(12.0)).fill(Color32::from_rgb(26, 29, 33)).rounding(4.0)).clicked() {
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
                    let in_meeting_bg = if in_meeting_active { Color32::from_rgb(2, 132, 199) } else { Color32::from_rgb(26, 29, 33) };
                    if ui.add(egui::Button::new(RichText::new(format!("In Meeting ({total_in_meeting})")).size(11.0).color(Color32::WHITE)).fill(in_meeting_bg).rounding(6.0)).clicked() {
                        app.roster_tab = RosterTab::InMeeting;
                    }

                    let waiting_active = app.roster_tab == RosterTab::WaitingRoom;
                    let waiting_bg = if waiting_active {
                        Color32::from_rgb(2, 132, 199)
                    } else if waiting_count > 0 {
                        Color32::from_rgb(45, 30, 10)
                    } else {
                        Color32::from_rgb(26, 29, 33)
                    };
                    let waiting_text_color = if waiting_count > 0 && !waiting_active {
                        Color32::from_rgb(251, 191, 36)
                    } else {
                        Color32::WHITE
                    };

                    let waiting_label = if waiting_count > 0 {
                        format!("⏳ Waiting ({waiting_count})")
                    } else {
                        "Waiting Room (0)".to_string()
                    };

                    if ui.add(egui::Button::new(RichText::new(waiting_label).size(11.0).color(waiting_text_color)).fill(waiting_bg).rounding(6.0)).clicked() {
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
            let mut kick_action: Option<Uuid> = None;

            if is_host && app.roster_tab == RosterTab::WaitingRoom {
                // --- WAITING ROOM TAB CONTENT ---
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Waiting in Lobby ({waiting_count})")).size(12.0).strong().color(Color32::from_rgb(251, 191, 36)));
                    if waiting_count > 0 {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("✓ Admit All").size(10.0).strong().color(Color32::WHITE)).fill(Color32::from_rgb(16, 185, 129)).rounding(4.0)).clicked() {
                                admit_all = true;
                            }
                        });
                    }
                });
                ui.add_space(8.0);

                if waiting_count == 0 {
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        ui.label(RichText::new("No participants currently waiting.").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                        ui.add_space(4.0);
                        ui.label(RichText::new("New joiners will appear here when the Waiting Room policy is enabled.").size(10.0).color(Color32::from_rgb(71, 85, 105)));
                    });
                } else {
                    ScrollArea::vertical().show(ui, |ui| {
                        let waiting_list = app.waiting_participants.clone();
                        for p in waiting_list {
                            egui::Frame::group(ui.style())
                                .fill(Color32::from_rgb(26, 29, 33))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
                                .rounding(6.0)
                                .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let initial = p.display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                        egui::Frame::group(ui.style())
                                            .fill(Color32::from_rgb(38, 42, 48))
                                            .stroke(Stroke::NONE)
                                            .rounding(12.0)
                                            .inner_margin(4.0)
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(initial).size(10.0).strong().color(Color32::WHITE));
                                            });

                                        ui.vertical(|ui| {
                                            ui.label(RichText::new(&p.display_name).strong().size(12.0).color(Color32::from_rgb(248, 250, 252)));
                                            if let Some(email) = &p.email {
                                                ui.label(RichText::new(email).size(10.0).color(Color32::from_rgb(148, 163, 184)));
                                            }
                                        });

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.add(egui::Button::new(RichText::new("✕ Reject").size(10.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(4.0)).clicked() {
                                                reject_action = Some(p.participant_id);
                                            }
                                            if ui.add(egui::Button::new(RichText::new("✓ Admit").size(10.0).strong().color(Color32::WHITE)).fill(Color32::from_rgb(16, 185, 129)).rounding(4.0)).clicked() {
                                                admit_action = Some(p.participant_id);
                                            }
                                        });
                                    });
                                });
                            ui.add_space(4.0);
                        }
                    });
                }
            } else {
                // --- IN-MEETING PARTICIPANTS TAB CONTENT ---
                ScrollArea::vertical().show(ui, |ui| {
                    // Local participant row
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(26, 29, 33))
                        .stroke(Stroke::NONE)
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let initial = app.user_display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                ui.label(RichText::new(format!("({initial})")).size(11.0).color(Color32::from_rgb(56, 189, 248)));
                                ui.label(RichText::new(&app.user_display_name).strong().size(12.0).color(Color32::from_rgb(248, 250, 252)));
                                ui.label(RichText::new("(You)").size(10.0).color(Color32::from_rgb(148, 163, 184)));

                                if app.my_role == "host" {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new("★ Host").size(10.0).strong().color(Color32::from_rgb(245, 158, 11)));
                                    });
                                }
                            });
                        });
                    ui.add_space(6.0);

                    // Remote participants
                    for p in &app.roster {
                        egui::Frame::group(ui.style())
                            .fill(Color32::from_rgb(26, 29, 33))
                            .stroke(Stroke::NONE)
                            .rounding(6.0)
                            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let initial = p.display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                    ui.label(RichText::new(format!("({initial})")).size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                    ui.label(RichText::new(&p.display_name).size(12.0).color(Color32::from_rgb(248, 250, 252)));

                                    if p.role == "host" {
                                        ui.label(RichText::new("★").size(10.0).color(Color32::from_rgb(245, 158, 11)));
                                    }

                                    if p.is_audio_muted {
                                        ui.label(RichText::new("🔇").size(10.0));
                                    }
                                    if p.is_hand_raised {
                                        ui.label(RichText::new("✋").size(10.0));
                                    }

                                    if is_host {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.add(egui::Button::new(RichText::new("Kick").size(10.0).color(Color32::from_rgb(254, 205, 211))).fill(Color32::from_rgb(225, 29, 72)).rounding(4.0)).clicked() {
                                                kick_action = Some(p.participant_id);
                                            }
                                            if ui.add(egui::Button::new(RichText::new("Mute").size(10.0).color(Color32::WHITE)).fill(Color32::from_rgb(38, 42, 48)).rounding(4.0)).clicked() {
                                                mute_action = Some(p.participant_id);
                                            }
                                        });
                                    }
                                });
                            });
                        ui.add_space(6.0);
                    }
                });
            }

            // Perform dispatched actions
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
            if let Some(id) = kick_action {
                app.host_kick_participant(id);
            }
        });
}
