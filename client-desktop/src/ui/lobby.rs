use egui::{Color32, RichText, Stroke, Ui, Vec2};
use crate::app::ConferApp;
use crate::media::filters::VideoFilter;
use crate::media::VirtualBackgroundMode;
use crate::ui::components::Components;
use crate::ui::theme::Theme;

pub fn render_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let available_rect = ui.available_rect_before_wrap();
    let avail_w = ui.available_width();

    // 1. Pristine luxury Obsidian canvas
    ui.painter().rect_filled(
        available_rect,
        0.0,
        Theme::CANVAS,
    );

    // Responsive container calculation
    let max_content_w = 1240.0_f32;
    let content_w = (avail_w - 48.0).clamp(320.0, max_content_w);
    let side_margin = ((avail_w - content_w) / 2.0).max(16.0);
    let is_compact = avail_w < 960.0;

    // Persona-specific theme color
    let persona_color = if app.user_email == "host@confer.local" {
        Theme::PRIMARY // Electric Blue
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
                            ui.label(RichText::new("⚡").size(14.0).color(Color32::WHITE));
                        });
                    ui.add_space(8.0);
                    ui.label(RichText::new("CONFER").size(20.0).strong().color(Theme::TEXT_PRIMARY));
                    ui.label(RichText::new("STUDIO").size(10.5).strong().color(Theme::PRIMARY_LIGHT));
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
                                Components::avatar_badge(ui, &user_initials, 22.0, 10.5, persona_color);
                                ui.label(RichText::new(&app.user_display_name).size(12.0).strong().color(Theme::TEXT_PRIMARY));
                                ui.label(RichText::new("•").size(9.0).color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(&app.user_email).size(11.0).color(Theme::TEXT_SECONDARY));
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
                                ui.label(RichText::new("WebRTC SFU Ready").size(11.0).strong().color(Color32::from_rgb(226, 232, 240)));
                                ui.label(RichText::new("•").size(9.0).color(Theme::TEXT_MUTED));
                                ui.label(RichText::new("<40MB Native").size(10.5).color(Theme::PRIMARY_LIGHT));
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
                        render_studio_viewfinder_card(app, ui, content_w, persona_color, &user_initials);
                        ui.add_space(16.0);
                        render_meeting_cards_column(app, ui, content_w);
                    });
                } else {
                    // --- EXPANDED VIEW (2-COLUMN GRID) ---
                    let left_col_w = (content_w * 0.54).max(380.0);
                    let right_col_w = (content_w - left_col_w - 24.0).max(340.0);

                    ui.vertical(|ui| {
                        ui.set_width(left_col_w);
                        render_studio_viewfinder_card(app, ui, left_col_w, persona_color, &user_initials);
                    });

                    ui.add_space(24.0);

                    ui.vertical(|ui| {
                        ui.set_width(right_col_w);
                        render_meeting_cards_column(app, ui, right_col_w);
                    });
                }

                ui.add_space(side_margin);
            });

            ui.add_space(32.0);
        });
    });
}

/// Renders the Left Column Studio Monitor & Device Controls
fn render_studio_viewfinder_card(
    app: &mut ConferApp,
    ui: &mut Ui,
    col_w: f32,
    persona_color: Color32,
    user_initials: &str,
) {
    let card_inner_w = (col_w - 40.0).max(100.0);

    Theme::card_frame(ui.style()).show(ui, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Studio Viewfinder").size(15.0).strong().color(Theme::TEXT_PRIMARY));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let status_text = if app.is_camera_off {
                    "📷 Video Paused"
                } else {
                    "● 720p HD • 30 FPS"
                };
                let status_color = if app.is_camera_off {
                    Theme::TEXT_MUTED
                } else {
                    Theme::EMERALD
                };
                let status_bg = if app.is_camera_off {
                    Theme::SURFACE_2
                } else {
                    Color32::from_rgb(6, 78, 59) // Deep Emerald background
                };
                egui::Frame::group(ui.style())
                    .fill(status_bg)
                    .stroke(Stroke::new(1.0_f32, if app.is_camera_off { Theme::BORDER_SUBTLE } else { Theme::EMERALD }))
                    .rounding(Theme::RADIUS_PILL)
                    .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(status_text).size(10.5).strong().color(status_color));
                    });
            });
        });

        ui.add_space(12.0);

        // 16:9 Video Canvas Surface with Studio Lens Reticle
        let viewport_h = (card_inner_w * 0.5625).max(140.0);

        egui::Frame::group(ui.style())
            .fill(Theme::CANVAS)
            .stroke(Stroke::new(1.5_f32, Theme::SURFACE_3))
            .rounding(Theme::RADIUS_MD)
            .inner_margin(0.0)
            .show(ui, |ui| {
                ui.set_width(card_inner_w);
                ui.set_height(viewport_h);

                if !app.is_camera_off {
                    if let Some(tex) = &app.local_video_texture {
                        ui.centered_and_justified(|ui| {
                            ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(card_inner_w, viewport_h)).rounding(Theme::RADIUS_MD));
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(viewport_h * 0.28);
                        Components::avatar_badge(ui, user_initials, 56.0, 22.0, persona_color);
                        ui.add_space(8.0);
                        ui.label(RichText::new("Camera is paused").size(12.5).color(Theme::TEXT_SECONDARY));
                    });
                }
            });

        ui.add_space(14.0);

        // Hardware Toggles with Colorized States & Hotkey Hints
        ui.horizontal(|ui| {
            let btn_w = ((card_inner_w - 12.0) / 3.0).max(80.0);

            // Camera Toggle
            let cam_bg = if app.is_camera_off { Theme::CRIMSON } else { Color32::from_rgb(16, 185, 129) };
            let cam_text = if app.is_camera_off { "📷 Cam Off" } else { "🎥 Cam Active" };
            if ui.add_sized(
                Vec2::new(btn_w, 34.0),
                egui::Button::new(RichText::new(cam_text).size(11.5).strong().color(Color32::WHITE)).fill(cam_bg).rounding(Theme::RADIUS_SM),
            ).on_hover_text("Toggle Camera (Ctrl+E)").clicked() {
                app.toggle_camera();
            }

            // Mic Toggle
            let mic_bg = if app.is_mic_muted { Theme::CRIMSON } else { Color32::from_rgb(16, 185, 129) };
            let mic_text = if app.is_mic_muted { "🔇 Mic Muted" } else { "🎙 Mic Active" };
            if ui.add_sized(
                Vec2::new(btn_w, 34.0),
                egui::Button::new(RichText::new(mic_text).size(11.5).strong().color(Color32::WHITE)).fill(mic_bg).rounding(Theme::RADIUS_SM),
            ).on_hover_text("Toggle Microphone (Ctrl+D)").clicked() {
                app.toggle_mic();
            }

            // AI Denoise Toggle
            let denoise_bg = if app.is_ai_denoise_enabled { Theme::PRIMARY } else { Theme::SURFACE_2 };
            let denoise_text = if app.is_ai_denoise_enabled { "⚡ Denoise ON" } else { "⚡ Denoise OFF" };
            if ui.add_sized(
                Vec2::new(btn_w, 34.0),
                egui::Button::new(RichText::new(denoise_text).size(11.5).strong().color(Color32::WHITE)).fill(denoise_bg).rounding(Theme::RADIUS_SM),
            ).on_hover_text("RNNoise 48kHz Neural Noise Suppression").clicked() {
                app.toggle_ai_denoise();
            }
        });

        ui.add_space(14.0);

        // Responsive Studio Audio VU Visualizer (3-Tier High-Contrast Color Bands)
        ui.horizontal(|ui| {
            ui.label(RichText::new("Mic Energy:").size(11.0).color(Theme::TEXT_SECONDARY));
            let level = if app.is_mic_muted { 0.0 } else { app.mic_test_level };

            let total_segments = 20;
            let active_segments = ((level * total_segments as f32).round() as usize).min(total_segments);
            let segment_gap = 3.0_f32;
            let available_meter_w = (card_inner_w - 85.0).max(40.0);
            let segment_w = ((available_meter_w - (total_segments as f32 - 1.0) * segment_gap) / total_segments as f32).max(2.0);

            for i in 0..total_segments {
                let is_lit = i < active_segments;
                let seg_color = if !is_lit {
                    Theme::SURFACE_2
                } else if i > 16 {
                    Color32::from_rgb(244, 63, 94) // Red Peak (#F43F5E)
                } else if i > 12 {
                    Color32::from_rgb(245, 158, 11) // Amber Mid-High (#F59E0B)
                } else {
                    Color32::from_rgb(16, 185, 129) // Emerald Normal (#10B981)
                };
                let seg_rect = ui.allocate_exact_size(Vec2::new(segment_w, 8.0), egui::Sense::hover()).0;
                ui.painter().rect_filled(seg_rect, 2.0, seg_color);
            }
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(10.0);

        // Visual Tone Filters with Wrapped Grid (No horizontal scroll friction)
        ui.label(RichText::new("Color Tone Preset").size(12.0).strong().color(Theme::TEXT_PRIMARY));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for filter in VideoFilter::all() {
                let is_active = app.active_filter == *filter;
                let (swatch_color, bg) = if is_active {
                    (Color32::WHITE, Theme::PRIMARY)
                } else {
                    (Color32::from_rgb(148, 163, 184), Theme::SURFACE_2)
                };
                let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };

                let dot = match filter {
                    VideoFilter::StudioGlow => "✨ ",
                    VideoFilter::WarmSunset => "🟠 ",
                    VideoFilter::CoolNordic => "🔵 ",
                    VideoFilter::NoirBw => "⚪ ",
                    VideoFilter::VibrantPop => "🟣 ",
                    VideoFilter::VignetteFocus => "🎯 ",
                    VideoFilter::VintageFilm => "🟤 ",
                    VideoFilter::None => "🌿 ",
                };
                let label = format!("{}{}", dot, filter.label());

                if ui.add(
                    egui::Button::new(RichText::new(label).size(11.0).strong().color(text_color))
                        .fill(bg)
                        .stroke(Stroke::new(1.0_f32, if is_active { swatch_color } else { Theme::BORDER_SUBTLE }))
                        .rounding(Theme::RADIUS_PILL),
                ).clicked() {
                    app.active_filter = *filter;
                }
            }
        });

        ui.add_space(12.0);

        // Background & Portrait Blur Selector (Wrapped Grid)
        ui.label(RichText::new("Studio Background & Blur").size(12.0).strong().color(Theme::TEXT_PRIMARY));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for bg in VirtualBackgroundMode::all() {
                let is_active = app.virtual_bg_mode == *bg;
                let btn_bg = if is_active { Theme::PRIMARY } else { Theme::SURFACE_2 };
                let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(203, 213, 225) };
                if ui.add(egui::Button::new(RichText::new(bg.label()).size(11.0).strong().color(text_color)).fill(btn_bg).rounding(Theme::RADIUS_PILL)).clicked() {
                    app.set_virtual_bg_mode(bg.clone());
                }
            }

            let is_custom = matches!(app.virtual_bg_mode, VirtualBackgroundMode::CustomImage(_));
            let custom_bg = if is_custom { Theme::PRIMARY } else { Theme::SURFACE_2 };
            if ui.add(egui::Button::new(RichText::new("📁 Custom Photo...").size(11.0).strong().color(Color32::WHITE)).fill(custom_bg).rounding(Theme::RADIUS_PILL)).clicked() {
                app.choose_custom_background();
            }
        });
    });
}

/// Renders the Profile Persona, Host Meeting, and Join Meeting Cards with Status Feedback
fn render_meeting_cards_column(app: &mut ConferApp, ui: &mut Ui, col_w: f32) {
    let card_inner_w = (col_w - 40.0).max(100.0);

    // Global Error Banner at top of Action Column if present
    if let Some(err) = &app.error_message {
        Components::error_banner(ui, err);
        ui.add_space(12.0);
    }

    // Card 1: Participant Identity with Colorized Persona Chips
    Theme::card_frame(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Participant Profile").size(15.0).strong().color(Theme::TEXT_PRIMARY));
        ui.add_space(4.0);
        ui.label(RichText::new("Select a pre-configured profile or customize your identity:").size(11.5).color(Theme::TEXT_SECONDARY));
        ui.add_space(10.0);

        // Quick Persona Switcher Chips with Dedicated Colors
        ui.horizontal(|ui| {
            let chip_w = ((card_inner_w - 12.0) / 3.0).max(60.0);

            // Host Chip (Electric Blue / Amber Star)
            let is_host = app.user_email == "host@confer.local";
            let host_bg = if is_host { Color32::from_rgb(2, 132, 199) } else { Theme::SURFACE_2 };
            if ui.add_sized(
                Vec2::new(chip_w, 32.0),
                egui::Button::new(RichText::new("★ Host (Dev)").size(11.0).strong().color(if is_host { Color32::WHITE } else { Color32::from_rgb(251, 191, 36) }))
                    .fill(host_bg)
                    .rounding(Theme::RADIUS_SM),
            ).clicked() {
                app.user_email = "host@confer.local".to_string();
                app.user_display_name = "Host User (Dev)".to_string();
            }

            // Alice Chip (Violet)
            let is_alice = app.user_email == "participant1@confer.local";
            let alice_bg = if is_alice { Color32::from_rgb(139, 92, 246) } else { Theme::SURFACE_2 };
            if ui.add_sized(
                Vec2::new(chip_w, 32.0),
                egui::Button::new(RichText::new("👩 Alice (Guest)").size(11.0).strong().color(if is_alice { Color32::WHITE } else { Color32::from_rgb(216, 180, 254) }))
                    .fill(alice_bg)
                    .rounding(Theme::RADIUS_SM),
            ).clicked() {
                app.user_email = "participant1@confer.local".to_string();
                app.user_display_name = "Alice (Dev)".to_string();
            }

            // Bob Chip (Teal)
            let is_bob = app.user_email == "participant2@confer.local";
            let bob_bg = if is_bob { Color32::from_rgb(13, 148, 136) } else { Theme::SURFACE_2 };
            if ui.add_sized(
                Vec2::new(chip_w, 32.0),
                egui::Button::new(RichText::new("👨 Bob (Guest)").size(11.0).strong().color(if is_bob { Color32::WHITE } else { Color32::from_rgb(153, 246, 228) }))
                    .fill(bob_bg)
                    .rounding(Theme::RADIUS_SM),
            ).clicked() {
                app.user_email = "participant2@confer.local".to_string();
                app.user_display_name = "Bob (Dev)".to_string();
            }
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Your Name:").size(11.5).strong().color(Color32::from_rgb(203, 213, 225)));
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

    // Card 2: Host New Instant Meeting (Vibrant Sapphire/Electric Blue)
    Theme::focused_card_frame(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Start New Meeting").size(15.0).strong().color(Color32::WHITE));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(12, 74, 110))
                    .stroke(Stroke::new(1.0_f32, Theme::PRIMARY_LIGHT))
                    .rounding(Theme::RADIUS_PILL)
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("HOST (Ctrl+Enter)").size(9.5).strong().color(Theme::PRIMARY_LIGHT));
                    });
            });
        });

        ui.add_space(4.0);
        ui.label(RichText::new("Create an instant encrypted room with WebRTC SFU media routing.").size(11.5).color(Theme::TEXT_SECONDARY));
        ui.add_space(10.0);

        // Meeting Title Input
        ui.label(RichText::new("Meeting Topic:").size(11.5).strong().color(Color32::from_rgb(203, 213, 225)));
        ui.add_space(3.0);
        ui.add_sized(
            Vec2::new(card_inner_w, 32.0),
            egui::TextEdit::singleline(&mut app.meeting_title_input)
                .hint_text("e.g. Sprint Planning, Architecture Review...")
                .margin(egui::Margin::symmetric(10.0, 6.0)),
        );

        ui.add_space(12.0);

        // Full-Width Start Meeting Action Button with Disabled Guard & Spinner
        ui.add_enabled_ui(!app.is_connecting, |ui| {
            let btn_text = if app.is_connecting {
                "⏳ Connecting to Server..."
            } else {
                "Start Meeting Now (Ctrl+Enter)"
            };

            if ui.add_sized(
                Vec2::new(card_inner_w, 38.0),
                Components::primary_button(btn_text, 13.0),
            ).clicked() {
                app.trigger_create_meeting();
            }
        });
    });

    ui.add_space(16.0);

    // Card 3: Join Meeting by 6-Character Code
    Theme::card_frame(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Join with Code").size(15.0).strong().color(Theme::TEXT_PRIMARY));
        ui.add_space(4.0);
        ui.label(RichText::new("Enter the 6-character room code provided by the meeting host:").size(11.5).color(Theme::TEXT_SECONDARY));
        ui.add_space(10.0);

        // Responsive Join Code Row
        ui.horizontal(|ui| {
            let btn_w = 110.0_f32;
            let input_w = (card_inner_w - btn_w - 10.0).max(80.0);

            let edit_res = ui.add_sized(
                Vec2::new(input_w, 36.0),
                egui::TextEdit::singleline(&mut app.join_code_input)
                    .hint_text("PIN (e.g. ABC123)")
                    .font(egui::FontId::monospace(14.0))
                    .margin(egui::Margin::symmetric(10.0, 6.0)),
            );
            app.join_code_input = app.join_code_input.to_uppercase();

            let can_join = !app.join_code_input.trim().is_empty() && !app.is_connecting;

            ui.add_enabled_ui(can_join, |ui| {
                let join_text = if app.is_connecting { "⏳ Joining..." } else { "Join Room" };
                if (ui.add_sized(
                    Vec2::new(btn_w, 36.0),
                    Components::primary_button(join_text, 12.5),
                ).clicked() || (edit_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && can_join {
                    app.trigger_join_meeting();
                }
            });
        });
    });
}
