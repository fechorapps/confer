use crate::app::ConferApp;
use crate::ui::theme::Theme;
use crate::ui::{captions, chat, controls, diagnostics, polls, roster, whiteboard};
use egui::{Color32, RichText, Stroke, Ui};

mod push_to_talk;
mod reactions;
mod safety_modals;
mod screen_share;
mod video_grid;

pub fn render_meeting_room(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // 1. Dark Obsidian Canvas Base
    ui.painter().rect_filled(
        full_rect,
        0.0,
        Theme::CANVAS, // Obsidian Base
    );

    // 2. Top Header Bar (Elevated Deep Zinc with Precision Bottom Stroke)
    egui::Frame::group(ui.style())
        .fill(Theme::SURFACE_1)
        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
        .rounding(0.0)
        .inner_margin(egui::Margin::symmetric(20.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left Brand & Room Information
                ui.horizontal(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(Theme::BORDER_ACTIVE)
                        .stroke(Stroke::NONE)
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡").size(12.0).color(Theme::ON_ACCENT));
                        });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("CONFER")
                            .size(15.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("•")
                            .size(10.0)
                            .color(Theme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new(&app.room_title)
                            .size(13.5)
                            .strong()
                            .color(Color32::from_rgb(226, 232, 240)),
                    );
                });

                // Join Code Pill (Click to copy with ephemeral '✓ Copied!' feedback)
                if let Some(code) = &app.current_join_code {
                    ui.add_space(8.0);
                    let now = ui.input(|i| i.time);
                    let is_recently_copied = app
                        .code_copied_time
                        .is_some_and(|t| now - t < 2.0);

                    let (code_text, code_col) = if is_recently_copied {
                        ("✓ Copied!".to_string(), Theme::EMERALD)
                    } else {
                        (format!("CODE: {code}"), Theme::PRIMARY_LIGHT)
                    };

                    let code_btn = ui.add(
                        egui::Button::new(
                            RichText::new(code_text)
                                .size(11.0)
                                .strong()
                                .font(egui::FontId::monospace(11.0))
                                .color(code_col),
                        )
                        .fill(Theme::SURFACE_2)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Theme::RADIUS_SM),
                    );
                    if code_btn
                        .on_hover_text("Click to copy room code to clipboard")
                        .clicked()
                    {
                        ui.output_mut(|o| o.copied_text = code.clone());
                        app.code_copied_time = Some(now);
                    }
                }

                // Room Lock Status Badge
                if app.is_room_locked {
                    ui.add_space(6.0);
                    egui::Frame::group(ui.style())
                        .fill(Color32::from_rgb(45, 30, 10))
                        .stroke(Stroke::new(1.0_f32, Theme::AMBER))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("🔒 LOCKED")
                                    .size(10.5)
                                    .strong()
                                    .color(Theme::AMBER_LIGHT),
                            );
                        });
                }

                // Right Status Hub (Clean Telemetry & Streaming Indicators)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // HUD Telemetry Toggle Button
                    let hud_bg = if app.show_diagnostics {
                        crate::ui::theme::Theme::PRIMARY
                    } else {
                        crate::ui::theme::Theme::SURFACE_2
                    };
                    let hud_fg = if app.show_diagnostics {
                        Theme::ON_ACCENT
                    } else {
                        Color32::WHITE
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("⚡ HUD")
                                    .size(11.0)
                                    .strong()
                                    .color(hud_fg),
                            )
                            .fill(hud_bg)
                            .rounding(crate::ui::theme::Theme::RADIUS_SM),
                        )
                        .on_hover_text("Toggle Real-Time Diagnostics HUD")
                        .clicked()
                    {
                        app.show_diagnostics = !app.show_diagnostics;
                    }

                    ui.add_space(8.0);

                    // Network Health Pill
                    egui::Frame::group(ui.style())
                        .fill(crate::ui::theme::Theme::SURFACE_2)
                        .stroke(Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE))
                        .rounding(crate::ui::theme::Theme::RADIUS_PILL)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("●")
                                        .size(8.0)
                                        .color(crate::ui::theme::Theme::EMERALD),
                                );
                                ui.label(
                                    RichText::new(format!("{}ms RTT", app.rtt_ms))
                                        .size(11.0)
                                        .strong()
                                        .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new("•")
                                        .size(9.0)
                                        .color(crate::ui::theme::Theme::TEXT_MUTED),
                                );
                                ui.label(
                                    RichText::new(format!("{:.1}% Loss", app.packet_loss_pct))
                                        .size(10.5)
                                        .color(crate::ui::theme::Theme::TEXT_SECONDARY),
                                );
                            });
                        });
                });
            });
        });

    // 3. Main Stage & Video Grid + Side Panels
    let available_height = (ui.available_height() - 76.0).max(100.0); // Reserve space for bottom floating dock
    ui.horizontal(|ui| {
        // Stage / Video Grid Container
        ui.vertical(|ui| {
            ui.set_height(available_height);

            if app.is_whiteboard_active {
                whiteboard::render_whiteboard(app, ui);
            } else if app.is_screen_sharing {
                screen_share::render_screen_share_stage(app, ui);
            } else {
                video_grid::render_video_grid(app, ui);
            }
        });

        // Optional Side Drawers
        if app.show_chat {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                chat::render_chat(app, ui);
            });
        } else if app.show_roster {
            ui.vertical(|ui| {
                ui.set_width(320.0);
                roster::render_roster(app, ui);
            });
        } else if app.show_polls {
            ui.vertical(|ui| {
                ui.set_width(340.0);
                polls::render_polls(app, ui);
            });
        }
    });

    // 4. Floating Overlay Elements
    reactions::render_reactions(app, ui, full_rect);
    push_to_talk::render_push_to_talk_indicator(app, ui, full_rect);

    if app.show_diagnostics {
        diagnostics::render_diagnostics(app, ui.ctx());
    }

    captions::render_captions(app, ui, full_rect);

    // 5. Floating Bottom Control Dock
    controls::render_controls(app, ui);

    // 6. Safety Modals
    safety_modals::render_safety_modals(app, ui, full_rect);
}
