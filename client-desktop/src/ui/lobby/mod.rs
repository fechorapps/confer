use crate::app::ConferApp;
use crate::ui::components::Components;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Stroke, Ui};

mod cockpit;
mod viewfinder;

pub fn render_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let available_rect = ui.available_rect_before_wrap();
    let avail_w = ui.available_width();

    // 1. Pristine luxury Obsidian canvas
    ui.painter().rect_filled(available_rect, 0.0, Theme::CANVAS);

    // Responsive container calculation
    let max_content_w = 1240.0_f32;
    let content_w = (avail_w - 48.0).clamp(320.0, max_content_w);
    let side_margin = ((avail_w - content_w) / 2.0).max(16.0);
    let is_compact = avail_w < 960.0;

    // Persona-specific theme color
    let persona_color = if app.user_email == "host@confer.local" {
        Theme::PRIMARY // Electric Blue (#0284C7)
    } else if app.user_email == "participant1@confer.local" {
        Color32::from_rgb(139, 92, 246) // Violet (Alice)
    } else {
        Color32::from_rgb(13, 148, 136) // Teal (Bob)
    };

    let user_initials = Components::extract_initials(&app.user_display_name);

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // ==========================================
            // TOP GLOBAL BRAND & STATUS HEADER
            // ==========================================
            ui.horizontal(|ui| {
                ui.add_space(side_margin);

                // Brand Mark & Logomark
                ui.horizontal(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(Theme::PRIMARY)
                        .stroke(Stroke::new(1.0_f32, Theme::PRIMARY_LIGHT))
                        .rounding(Theme::RADIUS_MD)
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡").size(14.0).color(Theme::ON_ACCENT));
                        });
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("CONFER")
                            .size(20.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new("STUDIO")
                            .size(10.5)
                            .strong()
                            .color(Theme::PRIMARY_LIGHT),
                    );
                });

                // Right Status Hub
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(side_margin);

                    // User Identity Badge with Persona Accent
                    egui::Frame::group(ui.style())
                        .fill(Theme::SURFACE_1)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Theme::RADIUS_PILL)
                        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                Components::avatar_badge(
                                    ui,
                                    &user_initials,
                                    22.0,
                                    10.5,
                                    persona_color,
                                );
                                ui.label(
                                    RichText::new(&app.user_display_name)
                                        .size(12.0)
                                        .strong()
                                        .color(Theme::TEXT_PRIMARY),
                                );
                                ui.label(RichText::new("•").size(9.0).color(Theme::TEXT_MUTED));
                                ui.label(
                                    RichText::new(&app.user_email)
                                        .size(11.0)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });
                        });

                    ui.add_space(10.0);

                    // Server Health Status Pill (Vibrant Emerald)
                    egui::Frame::group(ui.style())
                        .fill(Theme::SURFACE_1)
                        .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                        .rounding(Theme::RADIUS_PILL)
                        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("●").size(9.0).color(Theme::EMERALD));
                                ui.label(
                                    RichText::new("WebRTC SFU Ready")
                                        .size(11.0)
                                        .strong()
                                        .color(Color32::from_rgb(226, 232, 240)),
                                );
                                ui.label(RichText::new("•").size(9.0).color(Theme::TEXT_MUTED));
                                ui.label(
                                    RichText::new("<40MB Native")
                                        .size(10.5)
                                        .color(Theme::PRIMARY_LIGHT),
                                );
                            });
                        });
                });
            });

            ui.add_space(20.0);

            // ==========================================
            // RESPONSIVE COCKPIT CONTENT
            // ==========================================
            ui.horizontal(|ui| {
                ui.add_space(side_margin);

                if is_compact {
                    // --- COMPACT VIEW (VERTICAL STACK) ---
                    ui.vertical(|ui| {
                        ui.set_width(content_w);
                        viewfinder::render_studio_viewfinder_card(
                            app,
                            ui,
                            content_w,
                            persona_color,
                            &user_initials,
                        );
                        ui.add_space(16.0);
                        cockpit::render_meeting_cockpit_column(app, ui, content_w);
                    });
                } else {
                    // --- EXPANDED VIEW (2-COLUMN GRID) ---
                    let left_col_w = (content_w * 0.52).max(380.0);
                    let right_col_w = (content_w - left_col_w - 24.0).max(340.0);

                    ui.vertical(|ui| {
                        ui.set_width(left_col_w);
                        viewfinder::render_studio_viewfinder_card(
                            app,
                            ui,
                            left_col_w,
                            persona_color,
                            &user_initials,
                        );
                    });

                    ui.add_space(24.0);

                    ui.vertical(|ui| {
                        ui.set_width(right_col_w);
                        cockpit::render_meeting_cockpit_column(app, ui, right_col_w);
                    });
                }

                ui.add_space(side_margin);
            });

            ui.add_space(32.0);
        });
    });
}
