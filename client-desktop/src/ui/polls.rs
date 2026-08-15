use egui::{Color32, Rect, RichText, ScrollArea, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;

pub fn render_polls(app: &mut ConferApp, ui: &mut Ui) {
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 23))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            // --- Header ---
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📊 Live Polls & Voting")
                        .strong()
                        .size(14.0)
                        .color(Color32::from_rgb(248, 250, 252)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("✕").size(12.0))
                                .fill(Color32::from_rgb(26, 29, 33))
                                .rounding(4.0),
                        )
                        .clicked()
                    {
                        app.show_polls = false;
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // --- Toggle between List View and Creation Dialog ---
            if app.poll_creating {
                render_poll_creation_form(app, ui);
            } else {
                render_polls_list(app, ui);
            }
        });
}

fn render_polls_list(app: &mut ConferApp, ui: &mut Ui) {
    // Top Bar: "+ Create Poll" Button
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    RichText::new("+ Create New Poll")
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(2, 132, 199))
                .rounding(6.0),
            )
            .clicked()
        {
            app.poll_creating = true;
            if app.poll_create_options.is_empty() {
                app.poll_create_options = vec!["".to_string(), "".to_string()];
            }
        }
    });

    ui.add_space(8.0);

    let scroll_height = ui.available_height() - 10.0;
    ScrollArea::vertical()
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if app.polls.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("No active polls")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Create a poll to gather real-time feedback from participants.")
                            .size(11.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                });
                return;
            }

            let is_host = app.my_role == "host";
            let my_user_id = app.my_user_id;

            let mut poll_to_close: Option<Uuid> = None;
            let mut vote_to_submit: Option<(Uuid, Vec<usize>)> = None;

            // Render each poll card (newest first)
            for poll in app.polls.iter().rev() {
                let poll_id = poll.id;
                let is_creator = Some(poll.creator_id) == my_user_id;
                let has_voted = app
                    .user_poll_votes
                    .get(&poll_id)
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);

                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(26, 29, 33))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
                    .rounding(8.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        // --- Status & Badges ---
                        ui.horizontal(|ui| {
                            if poll.is_closed {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(45, 20, 20))
                                    .stroke(Stroke::NONE)
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("CLOSED")
                                                .size(9.0)
                                                .strong()
                                                .color(Color32::from_rgb(248, 113, 113)),
                                        );
                                    });
                            } else {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(16, 60, 35))
                                    .stroke(Stroke::NONE)
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("● LIVE")
                                                .size(9.0)
                                                .strong()
                                                .color(Color32::from_rgb(52, 211, 153)),
                                        );
                                    });
                            }

                            if poll.multi_choice {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(30, 41, 59))
                                    .stroke(Stroke::NONE)
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("Multiple Choice")
                                                .size(9.0)
                                                .color(Color32::from_rgb(148, 163, 184)),
                                        );
                                    });
                            }

                            if poll.is_anonymous {
                                egui::Frame::group(ui.style())
                                    .fill(Color32::from_rgb(30, 41, 59))
                                    .stroke(Stroke::NONE)
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("🔒 Anonymous")
                                                .size(9.0)
                                                .color(Color32::from_rgb(148, 163, 184)),
                                        );
                                    });
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(&poll.creator_name)
                                        .size(10.0)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            });
                        });

                        ui.add_space(6.0);

                        // --- Question Title ---
                        ui.label(
                            RichText::new(&poll.question)
                                .size(13.0)
                                .strong()
                                .color(Color32::from_rgb(248, 250, 252)),
                        );

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!(
                                "{} vote{}",
                                poll.total_votes,
                                if poll.total_votes == 1 { "" } else { "s" }
                            ))
                            .size(10.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                        );

                        ui.add_space(8.0);

                        let user_votes = app.user_poll_votes.get(&poll_id);

                        // --- Options rendering ---
                        if !poll.is_closed && !has_voted {
                            // User is actively voting
                            let selected_set = app
                                .poll_selected_options
                                .entry(poll_id)
                                .or_default();

                            for opt in &poll.options {
                                let is_selected = selected_set.contains(&opt.id);

                                if poll.multi_choice {
                                    let mut checked = is_selected;
                                    if ui.checkbox(&mut checked, RichText::new(&opt.text).size(12.0).color(Color32::WHITE)).changed() {
                                        if checked {
                                            selected_set.insert(opt.id);
                                        } else {
                                            selected_set.remove(&opt.id);
                                        }
                                    }
                                } else {
                                    if ui.radio(is_selected, RichText::new(&opt.text).size(12.0).color(Color32::WHITE)).clicked() {
                                        selected_set.clear();
                                        selected_set.insert(opt.id);
                                    }
                                }
                                ui.add_space(4.0);
                            }

                            ui.add_space(6.0);

                            // Submit Vote Button
                            let can_submit = !selected_set.is_empty();
                            let submit_btn = egui::Button::new(
                                RichText::new("Submit Vote")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(if can_submit {
                                Color32::from_rgb(2, 132, 199)
                            } else {
                                Color32::from_rgb(45, 50, 58)
                            })
                            .rounding(4.0);

                            if ui.add_enabled(can_submit, submit_btn).clicked() {
                                let selected: Vec<usize> = selected_set.iter().copied().collect();
                                vote_to_submit = Some((poll_id, selected));
                            }
                        } else {
                            // Results view with live percentage progress bars
                            for opt in &poll.options {
                                let pct = compute_option_percentage(opt.vote_count, poll.total_votes);

                                let user_voted_this = user_votes
                                    .map(|v| v.contains(&opt.id))
                                    .unwrap_or(false);

                                ui.horizontal(|ui| {
                                    let label_color = if user_voted_this {
                                        Color32::from_rgb(56, 189, 248)
                                    } else {
                                        Color32::from_rgb(226, 232, 240)
                                    };

                                    let prefix = if user_voted_this { "✓ " } else { "" };
                                    ui.label(
                                        RichText::new(format!("{}{}", prefix, opt.text))
                                            .size(11.0)
                                            .color(label_color),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "{} ({:.0}%)",
                                                opt.vote_count,
                                                pct * 100.0
                                            ))
                                            .size(10.0)
                                            .strong()
                                            .color(Color32::from_rgb(148, 163, 184)),
                                        );
                                    });
                                });

                                ui.add_space(2.0);

                                // Progress bar
                                let bar_width = ui.available_width();
                                let bar_height = 8.0;
                                let (rect, _) = ui.allocate_exact_size(
                                    Vec2::new(bar_width, bar_height),
                                    egui::Sense::hover(),
                                );

                                // Background track
                                ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(38, 42, 48));

                                // Filled portion
                                if pct > 0.0 {
                                    let fill_w = (bar_width * pct).max(6.0);
                                    let fill_rect = Rect::from_min_size(
                                        rect.min,
                                        Vec2::new(fill_w, bar_height),
                                    );
                                    let bar_color = if user_voted_this {
                                        Color32::from_rgb(2, 132, 199)
                                    } else {
                                        Color32::from_rgb(14, 165, 233)
                                    };
                                    ui.painter().rect_filled(fill_rect, 4.0, bar_color);
                                }

                                ui.add_space(6.0);
                            }
                        }

                        // --- Host / Creator Close Button ---
                        if (is_host || is_creator) && !poll.is_closed {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("🔒 End & Close Poll")
                                                .size(10.0)
                                                .color(Color32::from_rgb(254, 205, 211)),
                                        )
                                        .fill(Color32::from_rgb(159, 18, 57))
                                        .rounding(4.0),
                                    )
                                    .clicked()
                                {
                                    poll_to_close = Some(poll_id);
                                }
                            });
                        }
                    });

                ui.add_space(8.0);
            }

            // Execute any actions
            if let Some(id) = poll_to_close {
                app.close_poll(id);
            }
            if let Some((id, selected)) = vote_to_submit {
                app.vote_poll(id, selected);
            }
        });
}

fn render_poll_creation_form(app: &mut ConferApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(RichText::new("← Back").size(11.0).color(Color32::WHITE))
                    .fill(Color32::from_rgb(38, 42, 48))
                    .rounding(4.0),
            )
            .clicked()
        {
            app.poll_creating = false;
        }
        ui.label(
            RichText::new("New Poll")
                .size(13.0)
                .strong()
                .color(Color32::from_rgb(248, 250, 252)),
        );
    });

    ui.add_space(8.0);

    ScrollArea::vertical()
        .max_height(ui.available_height() - 10.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new("Question:")
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(148, 163, 184)),
            );
            ui.add(
                egui::TextEdit::multiline(&mut app.poll_create_question)
                    .desired_width(ui.available_width())
                    .desired_rows(2)
                    .hint_text("e.g. Should we adopt the new architecture?"),
            );

            ui.add_space(10.0);

            ui.label(
                RichText::new("Options:")
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(148, 163, 184)),
            );

            let mut option_to_remove: Option<usize> = None;
            let options_count = app.poll_create_options.len();

            for (idx, opt) in app.poll_create_options.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{}.", idx + 1))
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                    ui.add(
                        egui::TextEdit::singleline(opt)
                            .desired_width(ui.available_width() - 40.0)
                            .hint_text(format!("Option {}", idx + 1)),
                    );

                    if options_count > 2 {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("✕").size(10.0).color(Color32::from_rgb(248, 113, 113)),
                                )
                                .fill(Color32::from_rgb(38, 42, 48))
                                .rounding(4.0),
                            )
                            .clicked()
                        {
                            option_to_remove = Some(idx);
                        }
                    }
                });
                ui.add_space(4.0);
            }

            if let Some(idx) = option_to_remove {
                app.poll_create_options.remove(idx);
            }

            if app.poll_create_options.len() < 8 {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("+ Add Option")
                                .size(11.0)
                                .color(Color32::from_rgb(56, 189, 248)),
                        )
                        .fill(Color32::from_rgb(26, 29, 33))
                        .rounding(4.0),
                    )
                    .clicked()
                {
                    app.poll_create_options.push("".to_string());
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // Settings
            ui.checkbox(
                &mut app.poll_create_multi_choice,
                RichText::new("Allow multiple choices").size(11.0).color(Color32::WHITE),
            );
            ui.add_space(4.0);
            ui.checkbox(
                &mut app.poll_create_anonymous,
                RichText::new("Anonymous responses").size(11.0).color(Color32::WHITE),
            );

            ui.add_space(12.0);

            // Action Buttons
            ui.horizontal(|ui| {
                let valid_options: Vec<&String> = app
                    .poll_create_options
                    .iter()
                    .filter(|o| !o.trim().is_empty())
                    .collect();
                let can_launch = !app.poll_create_question.trim().is_empty() && valid_options.len() >= 2;

                let launch_btn = egui::Button::new(
                    RichText::new("🚀 Launch Poll")
                        .size(12.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(if can_launch {
                    Color32::from_rgb(2, 132, 199)
                } else {
                    Color32::from_rgb(45, 50, 58)
                })
                .rounding(6.0);

                if ui.add_enabled(can_launch, launch_btn).clicked() {
                    app.trigger_create_poll();
                }

                if ui
                    .add(
                        egui::Button::new(RichText::new("Cancel").size(12.0).color(Color32::WHITE))
                            .fill(Color32::from_rgb(38, 42, 48))
                            .rounding(6.0),
                    )
                    .clicked()
                {
                    app.poll_creating = false;
                }
            });
        });
}

pub fn compute_option_percentage(vote_count: u32, total_votes: u32) -> f32 {
    if total_votes == 0 {
        0.0
    } else {
        (vote_count as f32 / total_votes as f32).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::protocol::{PollDto, PollOptionDto};

    #[test]
    fn test_compute_option_percentage() {
        assert_eq!(compute_option_percentage(0, 0), 0.0);
        assert_eq!(compute_option_percentage(5, 10), 0.5);
        assert_eq!(compute_option_percentage(10, 10), 1.0);
        assert_eq!(compute_option_percentage(1, 3), 1.0 / 3.0);
    }

    #[test]
    fn test_poll_options_percentage_distribution() {
        let poll = PollDto {
            id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            creator_name: "Host".to_string(),
            question: "Choose meeting time:".to_string(),
            options: vec![
                PollOptionDto { id: 0, text: "9:00 AM".to_string(), vote_count: 2, voter_ids: vec![] },
                PollOptionDto { id: 1, text: "1:00 PM".to_string(), vote_count: 5, voter_ids: vec![] },
                PollOptionDto { id: 2, text: "4:00 PM".to_string(), vote_count: 3, voter_ids: vec![] },
            ],
            multi_choice: false,
            is_anonymous: true,
            is_closed: false,
            total_votes: 10,
            created_at: "2026-08-14T20:00:00Z".to_string(),
        };

        let p0 = compute_option_percentage(poll.options[0].vote_count, poll.total_votes);
        let p1 = compute_option_percentage(poll.options[1].vote_count, poll.total_votes);
        let p2 = compute_option_percentage(poll.options[2].vote_count, poll.total_votes);

        assert_eq!(p0, 0.2);
        assert_eq!(p1, 0.5);
        assert_eq!(p2, 0.3);
        assert!((p0 + p1 + p2 - 1.0).abs() < 0.0001);
    }
}
