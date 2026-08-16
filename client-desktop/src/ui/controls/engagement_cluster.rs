use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui, Vec2};

pub(super) fn render_engagement_cluster(app: &mut ConferApp, ui: &mut Ui, is_compact: bool) {
    let hand_bg = if app.is_hand_raised {
        Theme::AMBER
    } else {
        Theme::SURFACE_2
    };
    let hand_text = if app.is_hand_raised {
        "✋ Lower"
    } else {
        "✋ Hand"
    };
    if ui
        .add_sized(
            Vec2::new(0.0, 32.0),
            egui::Button::new(
                RichText::new(hand_text)
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(hand_bg)
            .rounding(Theme::RADIUS_PILL),
        )
        .on_hover_text("Raise / Lower Hand (Ctrl+H)")
        .clicked()
    {
        app.toggle_hand_raise();
    }

    // Emoji Reactions Popover (8 High-Fidelity Reactions)
    ui.menu_button(
        RichText::new("✨ React")
            .size(11.5)
            .strong()
            .color(Color32::WHITE),
        |ui| {
            ui.horizontal(|ui| {
                for emoji in ["👍", "❤️", "👏", "🎉", "🚀", "💡", "🔥", "💯"] {
                    if ui
                        .add(
                            egui::Button::new(RichText::new(emoji).size(17.0))
                                .fill(Theme::SURFACE_2)
                                .rounding(Theme::RADIUS_SM),
                        )
                        .clicked()
                    {
                        app.send_reaction(emoji);
                        ui.close_menu();
                    }
                }
            });
        },
    );

    // Progressive Overflow Tiering: Inline on wide screens, '⋯ Apps' on compact
    if !is_compact {
        let wb_bg = if app.is_whiteboard_active {
            Theme::PRIMARY
        } else {
            Theme::SURFACE_2
        };
        if ui
            .add_sized(
                Vec2::new(0.0, 32.0),
                egui::Button::new(
                    RichText::new("🖌 Board")
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(wb_bg)
                .rounding(Theme::RADIUS_PILL),
            )
            .on_hover_text("Collaborative Whiteboard (Ctrl+Shift+W)")
            .clicked()
        {
            app.toggle_whiteboard();
        }

        let polls_bg = if app.show_polls {
            Theme::PRIMARY
        } else {
            Theme::SURFACE_2
        };
        if ui
            .add_sized(
                Vec2::new(0.0, 32.0),
                egui::Button::new(
                    RichText::new("📊 Polls")
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(polls_bg)
                .rounding(Theme::RADIUS_PILL),
            )
            .on_hover_text("Live Polls (Ctrl+Shift+P)")
            .clicked()
        {
            app.toggle_polls();
        }

        let cc_bg = if app.is_captions_enabled {
            Theme::PRIMARY
        } else {
            Theme::SURFACE_2
        };
        let cc_text = if app.is_captions_enabled {
            "💬 CC On"
        } else {
            "💬 CC"
        };
        if ui
            .add_sized(
                Vec2::new(0.0, 32.0),
                egui::Button::new(
                    RichText::new(cc_text)
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(cc_bg)
                .rounding(Theme::RADIUS_PILL),
            )
            .on_hover_text("Live Speech-to-Text Subtitles (Ctrl+Shift+C)")
            .clicked()
        {
            app.toggle_captions();
        }
    } else {
        // Compact Apps Menu
        let has_active_app = app.is_whiteboard_active || app.show_polls || app.is_captions_enabled;
        let apps_text = if has_active_app {
            RichText::new("⋯ Apps (On) ▾")
                .size(11.5)
                .strong()
                .color(Theme::PRIMARY_LIGHT)
        } else {
            RichText::new("⋯ Apps ▾")
                .size(11.5)
                .strong()
                .color(Color32::WHITE)
        };

        ui.menu_button(apps_text, |ui| {
            ui.set_min_width(200.0);
            ui.label(
                RichText::new("Collaboration Tools")
                    .size(11.0)
                    .strong()
                    .color(Theme::PRIMARY_LIGHT),
            );
            ui.separator();

            let wb_label = if app.is_whiteboard_active {
                "✓ 🖌 Whiteboard (Active)"
            } else {
                "   🖌 Whiteboard"
            };
            if ui
                .selectable_label(app.is_whiteboard_active, wb_label)
                .clicked()
            {
                app.toggle_whiteboard();
                ui.close_menu();
            }

            let polls_label = if app.show_polls {
                "✓ 📊 Live Polls (Active)"
            } else {
                "   📊 Live Polls"
            };
            if ui.selectable_label(app.show_polls, polls_label).clicked() {
                app.toggle_polls();
                ui.close_menu();
            }

            let cc_label = if app.is_captions_enabled {
                "✓ 💬 Closed Captions (On)"
            } else {
                "   💬 Closed Captions"
            };
            if ui
                .selectable_label(app.is_captions_enabled, cc_label)
                .clicked()
            {
                app.toggle_captions();
                ui.close_menu();
            }
        });
    }
}
