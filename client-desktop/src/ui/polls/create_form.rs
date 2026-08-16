use egui::{Color32, RichText, ScrollArea, Ui};

use crate::app::ConferApp;
use crate::ui::theme::Theme;

pub(super) fn render_poll_creation_form(app: &mut ConferApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(RichText::new("← Back").size(11.0).color(Color32::WHITE))
                    .fill(Theme::SURFACE_3)
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
                .color(Theme::TEXT_PRIMARY),
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
                    .color(Theme::TEXT_SECONDARY),
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
                    .color(Theme::TEXT_SECONDARY),
            );

            let mut option_to_remove: Option<usize> = None;
            let options_count = app.poll_create_options.len();

            for (idx, opt) in app.poll_create_options.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{}.", idx + 1))
                            .size(11.0)
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add(
                        egui::TextEdit::singleline(opt)
                            .desired_width(ui.available_width() - 40.0)
                            .hint_text(format!("Option {}", idx + 1)),
                    );

                    if options_count > 2
                        && ui
                            .add(
                                egui::Button::new(
                                    RichText::new("✕")
                                        .size(10.0)
                                        .color(Color32::from_rgb(248, 113, 113)),
                                )
                                .fill(Theme::SURFACE_3)
                                .rounding(4.0),
                            )
                            .clicked()
                    {
                        option_to_remove = Some(idx);
                    }
                });
                ui.add_space(4.0);
            }

            if let Some(idx) = option_to_remove {
                app.poll_create_options.remove(idx);
            }

            if app.poll_create_options.len() < 8
                && ui
                    .add(
                        egui::Button::new(
                            RichText::new("+ Add Option")
                                .size(11.0)
                                .color(Theme::PRIMARY_LIGHT),
                        )
                        .fill(Theme::SURFACE_2)
                        .rounding(4.0),
                    )
                    .clicked()
            {
                app.poll_create_options.push("".to_string());
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // Settings
            ui.checkbox(
                &mut app.poll_create_multi_choice,
                RichText::new("Allow multiple choices")
                    .size(11.0)
                    .color(Color32::WHITE),
            );
            ui.add_space(4.0);
            ui.checkbox(
                &mut app.poll_create_anonymous,
                RichText::new("Anonymous responses")
                    .size(11.0)
                    .color(Color32::WHITE),
            );

            ui.add_space(12.0);

            // Action Buttons
            ui.horizontal(|ui| {
                let valid_options: Vec<&String> = app
                    .poll_create_options
                    .iter()
                    .filter(|o| !o.trim().is_empty())
                    .collect();
                let can_launch =
                    !app.poll_create_question.trim().is_empty() && valid_options.len() >= 2;

                let launch_btn =
                    egui::Button::new(RichText::new("🚀 Launch Poll").size(12.0).strong().color(
                        if can_launch {
                            Theme::ON_ACCENT
                        } else {
                            Color32::WHITE
                        },
                    ))
                    .fill(if can_launch {
                        Theme::BORDER_ACTIVE
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
                            .fill(Theme::SURFACE_3)
                            .rounding(6.0),
                    )
                    .clicked()
                {
                    app.poll_creating = false;
                }
            });
        });
}
