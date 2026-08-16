use crate::app::ConferApp;
use crate::ui::theme::Theme;
use crate::ui::watermark;
use egui::{Color32, RichText, Stroke, TextureHandle, Ui, Vec2};

pub(super) fn render_video_grid(app: &ConferApp, ui: &mut Ui) {
    let grid_rect = ui.available_rect_before_wrap();
    let count = app.roster.len() + 1;

    let (cols, rows) = match count {
        1 => (1, 1),
        2 => (2, 1),
        3..=4 => (2, 2),
        5..=6 => (3, 2),
        7..=9 => (3, 3),
        _ => (4, (count as f32 / 4.0).ceil() as usize),
    };

    let avail_size = ui.available_size();
    let gap = 12.0_f32;
    let tile_w = ((avail_size.x - (cols as f32 - 1.0) * gap) / cols as f32).max(120.0);
    let tile_h = ((avail_size.y - (rows as f32 - 1.0) * gap) / rows as f32).max(80.0);

    let mut tiles: Vec<TileProps<'_>> = Vec::with_capacity(count);

    // Local participant tile
    tiles.push(TileProps {
        width: tile_w,
        height: tile_h,
        name: &app.user_display_name,
        role: &app.my_role,
        is_audio_muted: app.is_mic_muted,
        is_video_muted: app.is_camera_off,
        is_sharing: app.is_screen_sharing,
        is_hand_raised: app.is_hand_raised,
        is_local: true,
        is_active_speaker: app
            .my_participant_id
            .is_some_and(|id| app.active_speaker_ids.contains(&id)),
        local_texture: app.local_video_texture.as_ref(),
    });

    // Remote participants
    for p in &app.roster {
        tiles.push(TileProps {
            width: tile_w,
            height: tile_h,
            name: &p.display_name,
            role: &p.role,
            is_audio_muted: p.is_audio_muted,
            is_video_muted: p.is_video_muted,
            is_sharing: p.is_screen_sharing,
            is_hand_raised: p.is_hand_raised,
            is_local: false,
            is_active_speaker: app.active_speaker_ids.contains(&p.participant_id),
            local_texture: app.remote_video_textures.get(&p.participant_id),
        });
    }

    egui::Grid::new("video_tiles_grid")
        .spacing([gap, gap])
        .show(ui, |ui| {
            for (idx, props) in tiles.iter().enumerate() {
                render_single_tile(ui, props);

                if (idx + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });

    if app.is_watermark_enabled || app.meeting_policy.watermark_enabled {
        watermark::render_watermark(ui, grid_rect, &app.user_display_name, &app.user_email);
    }
}

pub(super) struct TileProps<'a> {
    pub width: f32,
    pub height: f32,
    pub name: &'a str,
    pub role: &'a str,
    pub is_audio_muted: bool,
    pub is_video_muted: bool,
    pub is_sharing: bool,
    pub is_hand_raised: bool,
    pub is_local: bool,
    pub is_active_speaker: bool,
    pub local_texture: Option<&'a TextureHandle>,
}

pub(super) fn render_single_tile(ui: &mut Ui, props: &TileProps<'_>) {
    let stroke = if props.is_active_speaker {
        Stroke::new(2.5_f32, crate::ui::theme::Theme::EMERALD) // Emerald active speaker ring
    } else {
        Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE)
    };

    let bg_color = if props.is_video_muted {
        crate::ui::theme::Theme::SURFACE_1
    } else {
        Color32::from_rgb(5, 6, 8) // Deep letterbox canvas
    };

    egui::Frame::group(ui.style())
        .fill(bg_color)
        .stroke(stroke)
        .rounding(crate::ui::theme::Theme::RADIUS_LG)
        .inner_margin(0.0)
        .show(ui, |ui| {
            ui.set_width(props.width);
            ui.set_height(props.height);

            // Active Speaker 2-Pass Ambient Halo
            if props.is_active_speaker {
                let cell_rect = ui.min_rect();
                ui.painter().rect_stroke(
                    cell_rect.expand(2.0),
                    crate::ui::theme::Theme::RADIUS_LG + 2.0,
                    Stroke::new(1.5_f32, Color32::from_rgba_premultiplied(Theme::EMERALD.r(), Theme::EMERALD.g(), Theme::EMERALD.b(), 70)),
                );
            }

            if props.is_sharing {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.35);
                    ui.label(
                        RichText::new("🖥 Screen Share Active")
                            .size(15.0)
                            .strong()
                            .color(crate::ui::theme::Theme::PRIMARY_LIGHT),
                    );
                });
            } else if props.is_video_muted {
                ui.vertical_centered(|ui| {
                    ui.add_space((props.height * 0.28).max(10.0));
                    let initial = props
                        .name
                        .chars()
                        .next()
                        .unwrap_or('U')
                        .to_uppercase()
                        .to_string();
                    egui::Frame::group(ui.style())
                        .fill(crate::ui::theme::Theme::SURFACE_2)
                        .stroke(Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE))
                        .rounding(32.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(initial)
                                    .size(28.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                        });
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(props.name)
                            .size(12.5)
                            .strong()
                            .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                    );
                });
            } else if let Some(tex) = props.local_texture {
                // 16:9 Letterbox / Pillarbox Fit (No aspect distortion)
                let target_aspect = 16.0_f32 / 9.0_f32;
                let cell_w = (props.width - 2.0).max(40.0);
                let cell_h = (props.height - 2.0).max(40.0);
                let cell_aspect = cell_w / cell_h;

                let (draw_w, draw_h) = if cell_aspect > target_aspect {
                    (cell_h * target_aspect, cell_h)
                } else {
                    (cell_w, cell_w / target_aspect)
                };

                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::new(tex)
                            .fit_to_exact_size(Vec2::new(draw_w, draw_h))
                            .rounding(crate::ui::theme::Theme::RADIUS_MD),
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(props.height * 0.35);
                    ui.label(
                        RichText::new("🎥 Video Stream")
                            .size(13.5)
                            .color(crate::ui::theme::Theme::TEXT_SECONDARY),
                    );
                    if props.is_local {
                        ui.label(
                            RichText::new("(You)")
                                .size(11.0)
                                .color(crate::ui::theme::Theme::TEXT_MUTED),
                        );
                    }
                });
            }

            // Top-Right Raised Hand Pill Badge
            if props.is_hand_raised {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);
                            egui::Frame::group(ui.style())
                                .fill(crate::ui::theme::Theme::AMBER)
                                .stroke(Stroke::NONE)
                                .rounding(crate::ui::theme::Theme::RADIUS_PILL)
                                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("✋ Hand Raised")
                                            .size(10.5)
                                            .strong()
                                            .color(Color32::BLACK),
                                    );
                                });
                        });
                    });
                });
            }

            // Bottom-Left Floating Identity, Mic & Waveform Status Pill
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    egui::Frame::group(ui.style())
                        .fill(crate::ui::theme::Theme::SURFACE_DOCK)
                        .stroke(Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE))
                        .rounding(crate::ui::theme::Theme::RADIUS_SM)
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let mic_icon = if props.is_audio_muted { "🔇" } else { "🎙" };
                                let mic_color = if props.is_audio_muted {
                                    crate::ui::theme::Theme::CRIMSON
                                } else {
                                    crate::ui::theme::Theme::EMERALD
                                };
                                ui.label(RichText::new(mic_icon).size(11.0).color(mic_color));

                                // Active Speaker Audio Waveform Badge for Colorblind Accessibility
                                if props.is_active_speaker && !props.is_audio_muted {
                                    ui.label(
                                        RichText::new("ılı")
                                            .size(11.0)
                                            .strong()
                                            .color(crate::ui::theme::Theme::EMERALD),
                                    );
                                }

                                let name_label = if props.is_local {
                                    format!("{} (You)", props.name)
                                } else {
                                    props.name.to_string()
                                };
                                ui.label(
                                    RichText::new(name_label)
                                        .size(11.5)
                                        .strong()
                                        .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                                );

                                if props.role == "host" {
                                    ui.label(
                                        RichText::new("HOST")
                                            .size(9.0)
                                            .strong()
                                            .color(crate::ui::theme::Theme::PRIMARY_LIGHT),
                                    );
                                }
                            });
                        });
                });
            });
        });
}
