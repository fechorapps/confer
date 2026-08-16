use crate::app::{ConferApp, LobbyActionTab};
use crate::ui::components::Components;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Stroke, Ui, Vec2};

/// Renders the Unified High-Conviction Hero Action Cockpit Column
pub(super) fn render_meeting_cockpit_column(app: &mut ConferApp, ui: &mut Ui, col_w: f32) {
    let card_inner_w = (col_w - 40.0).max(100.0);

    // Global Error Banner at top of Action Column if present
    if let Some(err) = &app.error_message {
        Components::error_banner(ui, err);
        ui.add_space(12.0);
    }

    // Card 1: Participant Identity with Colorized Persona Chips
    Theme::card_frame(ui.style()).show(ui, |ui| {
        ui.label(
            RichText::new("Participant Profile")
                .size(15.0)
                .strong()
                .color(Theme::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Select a pre-configured profile or customize your identity:")
                .size(11.5)
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(10.0);

        // Quick Persona Switcher Chips with Dedicated Colors
        ui.horizontal(|ui| {
            let chip_w = ((card_inner_w - 12.0) / 3.0).max(60.0);

            // Host Chip (Electric Blue / Amber Star)
            let is_host = app.user_email == "host@confer.local";
            let host_bg = if is_host {
                Theme::BORDER_ACTIVE
            } else {
                Theme::SURFACE_2
            };
            if ui
                .add_sized(
                    Vec2::new(chip_w, 32.0),
                    egui::Button::new(RichText::new("★ Host (Dev)").size(11.0).strong().color(
                        if is_host {
                            Theme::ON_ACCENT
                        } else {
                            Theme::AMBER_LIGHT
                        },
                    ))
                    .fill(host_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .clicked()
            {
                app.user_email = "host@confer.local".to_string();
                app.user_display_name = "Host User (Dev)".to_string();
            }

            // Alice Chip (Violet)
            let is_alice = app.user_email == "participant1@confer.local";
            let alice_bg = if is_alice {
                Color32::from_rgb(139, 92, 246)
            } else {
                Theme::SURFACE_2
            };
            if ui
                .add_sized(
                    Vec2::new(chip_w, 32.0),
                    egui::Button::new(RichText::new("👩 Alice (Guest)").size(11.0).strong().color(
                        if is_alice {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(216, 180, 254)
                        },
                    ))
                    .fill(alice_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .clicked()
            {
                app.user_email = "participant1@confer.local".to_string();
                app.user_display_name = "Alice (Dev)".to_string();
            }

            // Bob Chip (Teal)
            let is_bob = app.user_email == "participant2@confer.local";
            let bob_bg = if is_bob {
                Color32::from_rgb(13, 148, 136)
            } else {
                Theme::SURFACE_2
            };
            if ui
                .add_sized(
                    Vec2::new(chip_w, 32.0),
                    egui::Button::new(RichText::new("👨 Bob (Guest)").size(11.0).strong().color(
                        if is_bob {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(153, 246, 228)
                        },
                    ))
                    .fill(bob_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .clicked()
            {
                app.user_email = "participant2@confer.local".to_string();
                app.user_display_name = "Bob (Dev)".to_string();
            }
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Your Name:")
                    .size(11.5)
                    .strong()
                    .color(Color32::from_rgb(203, 213, 225)),
            );
            let name_input_w = (card_inner_w - 90.0).max(100.0);
            ui.add_sized(
                Vec2::new(name_input_w, 28.0),
                egui::TextEdit::singleline(&mut app.user_display_name)
                    .hint_text("Enter display name...")
                    .margin(egui::Margin::symmetric(8.0, 4.0)),
            );
        });
    });

    ui.add_space(16.0);

    // Card 2: Bolder Segmented Hero Meeting Cockpit (Host vs Join)
    Theme::focused_card_frame(ui.style()).show(ui, |ui| {
        // High-Conviction Cockpit Segmented Switcher
        ui.horizontal(|ui| {
            let tab_w = ((card_inner_w - 8.0) / 2.0).max(100.0);

            // Tab 1: Host Instant Meeting
            let is_host_tab = app.lobby_action_tab == LobbyActionTab::Host;
            let host_tab_bg = if is_host_tab {
                Theme::PRIMARY
            } else {
                Theme::SURFACE_2
            };
            let host_tab_color = if is_host_tab {
                Theme::ON_ACCENT
            } else {
                Theme::TEXT_SECONDARY
            };
            if ui
                .add_sized(
                    Vec2::new(tab_w, 34.0),
                    egui::Button::new(
                        RichText::new("⚡ Start New Meeting")
                            .size(12.0)
                            .strong()
                            .color(host_tab_color),
                    )
                    .fill(host_tab_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .clicked()
            {
                app.lobby_action_tab = LobbyActionTab::Host;
            }

            // Tab 2: Join by PIN
            let is_join_tab = app.lobby_action_tab == LobbyActionTab::Join;
            let join_tab_bg = if is_join_tab {
                Theme::PRIMARY
            } else {
                Theme::SURFACE_2
            };
            let join_tab_color = if is_join_tab {
                Theme::ON_ACCENT
            } else {
                Theme::TEXT_SECONDARY
            };
            if ui
                .add_sized(
                    Vec2::new(tab_w, 34.0),
                    egui::Button::new(
                        RichText::new("🔗 Join with Code")
                            .size(12.0)
                            .strong()
                            .color(join_tab_color),
                    )
                    .fill(join_tab_bg)
                    .rounding(Theme::RADIUS_SM),
                )
                .clicked()
            {
                app.lobby_action_tab = LobbyActionTab::Join;
            }
        });

        ui.add_space(14.0);

        match app.lobby_action_tab {
            LobbyActionTab::Host => {
                // --- HOST MODE VIEW ---
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Instant Meeting Room")
                            .size(14.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        egui::Frame::group(ui.style())
                            .fill(Color32::from_rgb(12, 74, 110))
                            .stroke(Stroke::new(1.0_f32, Theme::PRIMARY_LIGHT))
                            .rounding(Theme::RADIUS_PILL)
                            .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("HOST")
                                        .size(9.5)
                                        .strong()
                                        .color(Theme::PRIMARY_LIGHT),
                                );
                            });
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new("Launch a private, encrypted SFU room with zero setup delay.")
                        .size(11.5)
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.add_space(12.0);

                ui.label(
                    RichText::new("Meeting Topic:")
                        .size(11.5)
                        .strong()
                        .color(Color32::from_rgb(203, 213, 225)),
                );
                ui.add_space(4.0);
                let topic_edit = ui.add_sized(
                    Vec2::new(card_inner_w, 34.0),
                    egui::TextEdit::singleline(&mut app.meeting_title_input)
                        .hint_text("e.g. Architecture Sync, Sprint Planning...")
                        .margin(egui::Margin::symmetric(10.0, 6.0)),
                );

                ui.add_space(14.0);

                ui.add_enabled_ui(!app.is_connecting, |ui| {
                    let btn_text = if app.is_connecting {
                        "⏳ Connecting to Server..."
                    } else {
                        "Start Meeting Now (Ctrl+Enter)"
                    };

                    if (ui
                        .add_sized(
                            Vec2::new(card_inner_w, 40.0),
                            Components::primary_button(btn_text, 13.5),
                        )
                        .clicked()
                        || (topic_edit.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !app.is_connecting
                    {
                        app.trigger_create_meeting();
                    }
                });

                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("🔒 SFrame Zero-Knowledge E2EE • WebRTC SFU")
                            .size(10.5)
                            .color(Theme::TEXT_MUTED),
                    );
                });
            }
            LobbyActionTab::Join => {
                // --- JOIN MODE VIEW ---
                ui.label(
                    RichText::new("Enter Meeting PIN")
                        .size(14.0)
                        .strong()
                        .color(Theme::TEXT_PRIMARY),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Enter the 6-character room code shared by your meeting host:")
                        .size(11.5)
                        .color(Theme::TEXT_SECONDARY),
                );
                ui.add_space(12.0);

                ui.label(
                    RichText::new("Room Code:")
                        .size(11.5)
                        .strong()
                        .color(Color32::from_rgb(203, 213, 225)),
                );
                ui.add_space(4.0);

                let edit_res = ui.add_sized(
                    Vec2::new(card_inner_w, 38.0),
                    egui::TextEdit::singleline(&mut app.join_code_input)
                        .hint_text("PIN (e.g. ABC123)")
                        .font(egui::FontId::monospace(15.0))
                        .margin(egui::Margin::symmetric(12.0, 8.0)),
                );
                app.join_code_input = app.join_code_input.to_uppercase();

                ui.add_space(14.0);

                let can_join = !app.join_code_input.trim().is_empty() && !app.is_connecting;

                ui.add_enabled_ui(can_join, |ui| {
                    let join_text = if app.is_connecting {
                        "⏳ Joining Meeting..."
                    } else {
                        "Join Room (Enter)"
                    };
                    if (ui
                        .add_sized(
                            Vec2::new(card_inner_w, 40.0),
                            Components::primary_button(join_text, 13.5),
                        )
                        .clicked()
                        || (edit_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                        && can_join
                    {
                        app.trigger_join_meeting();
                    }
                });

                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Ask the host for the 6-character room code")
                            .size(10.5)
                            .color(Theme::TEXT_MUTED),
                    );
                });
            }
        }
    });
}
