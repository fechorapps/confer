use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};

pub fn render_chat(app: &mut ConferApp, ui: &mut Ui) {
    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_1)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(Theme::RADIUS_MD)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("In-Call Messages")
                        .strong()
                        .size(14.0)
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("✕")
                                    .size(12.0)
                                    .color(Theme::TEXT_SECONDARY),
                            )
                            .fill(Theme::SURFACE_2)
                            .rounding(Theme::RADIUS_SM),
                        )
                        .clicked()
                    {
                        app.show_chat = false;
                    }
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Policy Notice if Chat is Disabled by Host
            let is_chat_allowed = app.meeting_policy.allow_chat || app.my_role == "host";
            if !is_chat_allowed {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(45, 30, 10))
                    .stroke(Stroke::new(1.0_f32, Theme::AMBER))
                    .rounding(Theme::RADIUS_SM)
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🔒 In-call chat is restricted by the host.")
                                    .size(11.0)
                                    .color(Theme::AMBER),
                            );
                        });
                    });
                ui.add_space(6.0);
            }

            // Messages Scroll Area
            let scroll_height = ui.available_height() - 50.0;
            ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if app.chat_messages.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(30.0);
                            ui.label(
                                RichText::new("No messages yet")
                                    .size(13.0)
                                    .color(Theme::TEXT_SECONDARY),
                            );
                            ui.label(
                                RichText::new("Messages sent during the call appear here")
                                    .size(11.0)
                                    .color(Theme::TEXT_MUTED),
                            );
                        });
                    }

                    for msg in &app.chat_messages {
                        let is_me = app.my_participant_id == Some(msg.from_id);
                        let bubble_bg = if is_me {
                            Theme::PRIMARY
                        } else {
                            Theme::SURFACE_2
                        };
                        let name_color = if is_me {
                            Color32::from_rgb(224, 242, 254)
                        } else {
                            Theme::PRIMARY_LIGHT
                        };

                        egui::Frame::group(ui.style())
                            .fill(bubble_bg)
                            .stroke(Stroke::NONE)
                            .rounding(Theme::RADIUS_SM)
                            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&msg.from_name)
                                            .strong()
                                            .size(11.0)
                                            .color(name_color),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(&msg.sent_at)
                                                    .size(10.0)
                                                    .color(Color32::from_rgb(203, 213, 225)),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(2.0);
                                ui.label(RichText::new(&msg.body).size(12.0).color(Color32::WHITE));
                            });
                        ui.add_space(6.0);
                    }
                });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(6.0);

            // Input Bar
            ui.add_enabled_ui(is_chat_allowed, |ui| {
                ui.horizontal(|ui| {
                    let text_edit = ui.add(
                        egui::TextEdit::singleline(&mut app.chat_input)
                            .desired_width(220.0)
                            .hint_text(if is_chat_allowed {
                                "Type a message..."
                            } else {
                                "Chat disabled by host"
                            }),
                    );
                    let send_clicked = ui
                        .add(
                            egui::Button::new(
                                RichText::new("Send")
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(if is_chat_allowed {
                                Theme::PRIMARY
                            } else {
                                Theme::SURFACE_3
                            })
                            .rounding(Theme::RADIUS_SM),
                        )
                        .clicked();

                    if is_chat_allowed
                        && (send_clicked
                            || (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !app.chat_input.trim().is_empty()
                    {
                        app.send_chat();
                        text_edit.request_focus();
                    }
                });
            });
        });
}
