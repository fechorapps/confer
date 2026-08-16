use egui::{Color32, Rect, RichText, ScrollArea, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;
use crate::ui::theme::Theme;

use super::model::compute_option_percentage;

pub(super) fn render_polls_list(app: &mut ConferApp, ui: &mut Ui) {
    // Top Bar: "+ Create Poll" Button
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    RichText::new("+ Create New Poll")
                        .size(12.0)
                        .strong()
                        .color(Theme::ON_ACCENT),
                )
                .fill(Theme::BORDER_ACTIVE)
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
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(
                            "Create a poll to gather real-time feedback from participants.",
                        )
                        .size(11.0)
                        .color(Theme::TEXT_MUTED),
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
                    .fill(Theme::SURFACE_2)
                    .stroke(Stroke::new(1.0_f32, Theme::SURFACE_3))
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
                                                .color(Theme::EMERALD_LIGHT),
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
                                                .color(Theme::TEXT_SECONDARY),
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
                                                .color(Theme::TEXT_SECONDARY),
                                        );
                                    });
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        RichText::new(&poll.creator_name)
                                            .size(10.0)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // --- Question Title ---
                        ui.label(
                            RichText::new(&poll.question)
                                .size(13.0)
                                .strong()
                                .color(Theme::TEXT_PRIMARY),
                        );

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!(
                                "{} vote{}",
                                poll.total_votes,
                                if poll.total_votes == 1 { "" } else { "s" }
                            ))
                            .size(10.0)
                            .color(Theme::TEXT_SECONDARY),
                        );

                        ui.add_space(8.0);

                        let user_votes = app.user_poll_votes.get(&poll_id);

                        // --- Options rendering ---
                        if !poll.is_closed && !has_voted {
                            // User is actively voting
                            let selected_set =
                                app.poll_selected_options.entry(poll_id).or_default();

                            for opt in &poll.options {
                                let is_selected = selected_set.contains(&opt.id);

                                if poll.multi_choice {
                                    let mut checked = is_selected;
                                    if ui
                                        .checkbox(
                                            &mut checked,
                                            RichText::new(&opt.text)
                                                .size(12.0)
                                                .color(Color32::WHITE),
                                        )
                                        .changed()
                                    {
                                        if checked {
                                            selected_set.insert(opt.id);
                                        } else {
                                            selected_set.remove(&opt.id);
                                        }
                                    }
                                } else if ui
                                    .radio(
                                        is_selected,
                                        RichText::new(&opt.text).size(12.0).color(Color32::WHITE),
                                    )
                                    .clicked()
                                {
                                    selected_set.clear();
                                    selected_set.insert(opt.id);
                                }
                                ui.add_space(4.0);
                            }

                            ui.add_space(6.0);

                            // Submit Vote Button
                            let can_submit = !selected_set.is_empty();
                            let submit_btn = egui::Button::new(
                                RichText::new("Submit Vote").size(11.0).strong().color(
                                    if can_submit {
                                        Theme::ON_ACCENT
                                    } else {
                                        Color32::WHITE
                                    },
                                ),
                            )
                            .fill(if can_submit {
                                Theme::BORDER_ACTIVE
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
                                let pct =
                                    compute_option_percentage(opt.vote_count, poll.total_votes);

                                let user_voted_this =
                                    user_votes.map(|v| v.contains(&opt.id)).unwrap_or(false);

                                ui.horizontal(|ui| {
                                    let label_color = if user_voted_this {
                                        Theme::PRIMARY_LIGHT
                                    } else {
                                        Color32::from_rgb(226, 232, 240)
                                    };

                                    let prefix = if user_voted_this { "✓ " } else { "" };
                                    ui.label(
                                        RichText::new(format!("{}{}", prefix, opt.text))
                                            .size(11.0)
                                            .color(label_color),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} ({:.0}%)",
                                                    opt.vote_count,
                                                    pct * 100.0
                                                ))
                                                .size(10.0)
                                                .strong()
                                                .color(Theme::TEXT_SECONDARY),
                                            );
                                        },
                                    );
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
                                ui.painter().rect_filled(rect, 4.0, Theme::SURFACE_3);

                                // Filled portion
                                if pct > 0.0 {
                                    let fill_w = (bar_width * pct).max(6.0);
                                    let fill_rect = Rect::from_min_size(
                                        rect.min,
                                        Vec2::new(fill_w, bar_height),
                                    );
                                    let bar_color = if user_voted_this {
                                        Theme::BORDER_ACTIVE
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

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("🔒 End & Close Poll")
                                                    .size(10.0)
                                                    .color(Theme::CRIMSON_LIGHT),
                                            )
                                            .fill(Color32::from_rgb(159, 18, 57))
                                            .rounding(4.0),
                                        )
                                        .clicked()
                                    {
                                        poll_to_close = Some(poll_id);
                                    }
                                },
                            );
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
