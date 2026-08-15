use crate::app::ConferApp;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};

pub fn render_chat(app: &mut ConferApp, ui: &mut Ui) {
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(18, 20, 23))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("In-Call Messages")
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
                        app.show_chat = false;
                    }
                });
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

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
                                    .color(Color32::from_rgb(148, 163, 184)),
                            );
                            ui.label(
                                RichText::new("Messages sent during the call appear here")
                                    .size(11.0)
                                    .color(Color32::from_rgb(100, 116, 139)),
                            );
                        });
                    }

                    for msg in &app.chat_messages {
                        let is_me = app.my_participant_id == Some(msg.from_id);
                        let bubble_bg = if is_me {
                            Color32::from_rgb(2, 132, 199)
                        } else {
                            Color32::from_rgb(26, 29, 33)
                        };
                        let name_color = if is_me {
                            Color32::from_rgb(224, 242, 254)
                        } else {
                            Color32::from_rgb(56, 189, 248)
                        };

                        egui::Frame::group(ui.style())
                            .fill(bubble_bg)
                            .stroke(Stroke::NONE)
                            .rounding(6.0)
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
            ui.horizontal(|ui| {
                let text_edit = ui.add(
                    egui::TextEdit::singleline(&mut app.chat_input)
                        .desired_width(220.0)
                        .hint_text("Type a message..."),
                );
                let send_clicked = ui
                    .add(
                        egui::Button::new(RichText::new("Send").strong().color(Color32::WHITE))
                            .fill(Color32::from_rgb(2, 132, 199))
                            .rounding(6.0),
                    )
                    .clicked();

                if (send_clicked
                    || (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                    && !app.chat_input.trim().is_empty()
                {
                    app.send_chat();
                    text_edit.request_focus();
                }
            });
        });
}
