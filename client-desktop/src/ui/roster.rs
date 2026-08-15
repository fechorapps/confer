use egui::{Color32, RichText, ScrollArea, Stroke, Ui};
use crate::app::ConferApp;

pub fn render_roster(app: &mut ConferApp, ui: &mut Ui) {
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 23))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Participants ({})", app.roster.len() + 1)).strong().size(14.0).color(Color32::from_rgb(248, 250, 252)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("✕").size(12.0)).fill(Color32::from_rgb(26, 29, 33)).rounding(4.0)).clicked() {
                        app.show_roster = false;
                    }
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let is_host = app.my_role == "host";

            // Host Moderation Bar
            if is_host {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(26, 29, 33))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let lock_label = if app.is_room_locked { "🔓 Unlock Room" } else { "🔒 Lock Room" };
                            let lock_bg = if app.is_room_locked { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(38, 42, 48) };
                            if ui.add(egui::Button::new(RichText::new(lock_label).size(11.0).color(Color32::WHITE)).fill(lock_bg).rounding(4.0)).clicked() {
                                app.toggle_room_lock();
                            }
                        });
                    });
                ui.add_space(8.0);
            }

            let mut mute_action = None;
            let mut kick_action = None;

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

            if let Some(id) = mute_action {
                app.host_mute_participant(id);
            }
            if let Some(id) = kick_action {
                app.host_kick_participant(id);
            }
        });
}
