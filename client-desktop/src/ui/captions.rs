use crate::app::ConferApp;
use crate::ui::theme::Theme;
use crate::sdk::protocol::CaptionChunkDto;
use egui::{Color32, Pos2, Rect, RichText, Stroke, Ui};

/// Auto-fade duration in seconds for live caption bubbles (10s for high readability)
pub const CAPTION_FADE_DURATION_SECS: f32 = 10.0;
/// Maximum number of active caption bubbles displayed simultaneously
pub const MAX_VISIBLE_CAPTIONS: usize = 3;

/// Helper function to get initials from a display name
pub fn get_speaker_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.is_empty() {
        return "?".to_string();
    }
    if parts.len() == 1 {
        let first = parts[0].chars().next().unwrap_or('?');
        return first.to_uppercase().to_string();
    }
    let first = parts[0].chars().next().unwrap_or('?');
    let last = parts[parts.len() - 1].chars().next().unwrap_or('?');
    format!("{}{}", first.to_uppercase(), last.to_uppercase())
}

/// Helper function to compute deterministic color from speaker name
pub fn get_speaker_color(name: &str) -> Color32 {
    let hash: u32 = name
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_add(b as u32).wrapping_mul(31));
    match hash % 6 {
        0 => Theme::PRIMARY_LIGHT,  // Sky Blue
        1 => Theme::EMERALD_LIGHT,  // Emerald
        2 => Theme::AMBER_LIGHT,  // Amber
        3 => Color32::from_rgb(167, 139, 250), // Violet
        4 => Color32::from_rgb(244, 114, 182), // Pink
        _ => Color32::from_rgb(94, 234, 212),  // Teal
    }
}

/// Renders the floating live captions overlay above the bottom dock
pub fn render_captions(app: &mut ConferApp, ui: &mut Ui, stage_rect: Rect) {
    if !app.is_captions_enabled || app.active_captions.is_empty() {
        return;
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    // Filter out expired captions older than CAPTION_FADE_DURATION_SECS
    let max_age_ms = (CAPTION_FADE_DURATION_SECS * 1000.0) as u64;
    app.active_captions.retain(|cap| {
        if cap.timestamp_ms == 0 || now_ms == 0 {
            true
        } else {
            now_ms.saturating_sub(cap.timestamp_ms) < max_age_ms
        }
    });

    if app.active_captions.is_empty() {
        return;
    }

    let total_captions = app.active_captions.len();
    let visible_slice = if total_captions > MAX_VISIBLE_CAPTIONS {
        &app.active_captions[total_captions - MAX_VISIBLE_CAPTIONS..]
    } else {
        &app.active_captions[..]
    };

    let panel_width = (stage_rect.width() * 0.75).clamp(420.0, 780.0);
    let dock_height_offset = 88.0; // Clear the bottom floating dock pill
    let bottom_anchor_y = stage_rect.max.y - dock_height_offset;
    let center_x = stage_rect.center().x;

    let overlay_pos = Pos2::new(center_x - panel_width * 0.5, bottom_anchor_y - 120.0);

    let area = egui::Area::new(egui::Id::new("floating_live_captions_overlay"))
        .fixed_pos(overlay_pos)
        .order(egui::Order::Foreground)
        .interactable(false);

    area.show(ui.ctx(), |ui| {
        ui.set_max_width(panel_width);

        egui::Frame::group(ui.style())
            .fill(Color32::from_rgba_premultiplied(15, 17, 21, 235))
            .stroke(Stroke::new(
                1.0_f32,
                Color32::from_rgba_premultiplied(255, 255, 255, 25),
            ))
            .rounding(14.0)
            .inner_margin(egui::Margin::symmetric(16.0, 10.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;

                    for cap in visible_slice {
                        render_caption_chunk(ui, cap, now_ms);
                    }
                });
            });
    });
}

fn render_caption_chunk(ui: &mut Ui, cap: &CaptionChunkDto, now_ms: u64) {
    let speaker_color = get_speaker_color(&cap.speaker_name);
    let initials = get_speaker_initials(&cap.speaker_name);

    // Calculate auto-fade alpha
    let age_ms = if now_ms > 0 && cap.timestamp_ms > 0 {
        now_ms.saturating_sub(cap.timestamp_ms)
    } else {
        0
    };
    let fade_progress = (age_ms as f32 / (CAPTION_FADE_DURATION_SECS * 1000.0)).clamp(0.0, 1.0);
    let alpha_factor = if fade_progress > 0.7 {
        ((1.0 - fade_progress) / 0.3).clamp(0.2, 1.0)
    } else {
        1.0
    };

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        // Speaker Avatar Pill
        let pill_bg = Color32::from_rgba_premultiplied(
            (speaker_color.r() as f32 * 0.25 * alpha_factor) as u8,
            (speaker_color.g() as f32 * 0.25 * alpha_factor) as u8,
            (speaker_color.b() as f32 * 0.25 * alpha_factor) as u8,
            (220.0 * alpha_factor) as u8,
        );

        egui::Frame::group(ui.style())
            .fill(pill_bg)
            .stroke(Stroke::new(
                1.0_f32,
                Color32::from_rgba_premultiplied(
                    speaker_color.r(),
                    speaker_color.g(),
                    speaker_color.b(),
                    (180.0 * alpha_factor) as u8,
                ),
            ))
            .rounding(10.0)
            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(initials)
                            .size(10.0)
                            .strong()
                            .color(speaker_color),
                    );
                    ui.label(
                        RichText::new(&cap.speaker_name)
                            .size(11.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                });
            });

        // Language tag if not en-US
        if !cap.language.is_empty() && cap.language != "en-US" && cap.language != "en" {
            let lang_tag = cap.language.to_uppercase();
            ui.label(
                RichText::new(format!("[{lang_tag}]"))
                    .size(10.0)
                    .color(Theme::TEXT_SECONDARY),
            );
        }

        // Subtitle Text
        let text_color = if cap.is_final {
            Color32::from_rgba_premultiplied(
                (248.0 * alpha_factor) as u8,
                (250.0 * alpha_factor) as u8,
                (252.0 * alpha_factor) as u8,
                (255.0 * alpha_factor) as u8,
            )
        } else {
            Color32::from_rgba_premultiplied(
                (148.0 * alpha_factor) as u8,
                (163.0 * alpha_factor) as u8,
                (184.0 * alpha_factor) as u8,
                (220.0 * alpha_factor) as u8,
            )
        };

        if cap.is_final {
            ui.label(RichText::new(&cap.text).size(13.5).color(text_color));
        } else {
            // Typing / interim indicator
            ui.label(
                RichText::new(format!("{} ⚡", cap.text))
                    .size(13.5)
                    .italics()
                    .color(text_color),
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_speaker_initials_extraction() {
        assert_eq!(get_speaker_initials("Host User"), "HU");
        assert_eq!(get_speaker_initials("Alice"), "A");
        assert_eq!(get_speaker_initials("Dr. Sarah Jane Connor"), "DC");
        assert_eq!(get_speaker_initials(""), "?");
    }

    #[test]
    fn test_speaker_color_determinism() {
        let col1 = get_speaker_color("Host User");
        let col2 = get_speaker_color("Host User");
        assert_eq!(col1, col2);

        let col_alice = get_speaker_color("Alice");
        assert_eq!(col_alice, get_speaker_color("Alice"));
    }

    #[test]
    fn test_caption_chunk_creation_and_fields() {
        let id = Uuid::new_v4();
        let chunk = CaptionChunkDto {
            participant_id: id,
            speaker_name: "Elena".to_string(),
            text: "Testing speech to text live subtitles".to_string(),
            is_final: false,
            language: "en-US".to_string(),
            timestamp_ms: 1723680000000,
        };

        assert_eq!(chunk.participant_id, id);
        assert_eq!(chunk.speaker_name, "Elena");
        assert_eq!(chunk.text, "Testing speech to text live subtitles");
        assert!(!chunk.is_final);
        assert_eq!(chunk.language, "en-US");
    }

    #[test]
    fn test_caption_interim_update_logic() {
        let id = Uuid::new_v4();
        let mut captions: Vec<CaptionChunkDto> = Vec::new();

        // 1. Initial interim chunk
        let chunk1 = CaptionChunkDto {
            participant_id: id,
            speaker_name: "Alice".to_string(),
            text: "Hello".to_string(),
            is_final: false,
            language: "en-US".to_string(),
            timestamp_ms: 1000,
        };
        captions.push(chunk1);
        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].text, "Hello");

        // 2. Updated interim chunk from same speaker
        let chunk2 = CaptionChunkDto {
            participant_id: id,
            speaker_name: "Alice".to_string(),
            text: "Hello everyone".to_string(),
            is_final: false,
            language: "en-US".to_string(),
            timestamp_ms: 1500,
        };
        if let Some(existing) = captions
            .iter_mut()
            .rev()
            .find(|c| c.participant_id == chunk2.participant_id && !c.is_final)
        {
            existing.text = chunk2.text;
            existing.is_final = chunk2.is_final;
        } else {
            captions.push(chunk2);
        }
        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].text, "Hello everyone");

        // 3. Finalized chunk
        let chunk3 = CaptionChunkDto {
            participant_id: id,
            speaker_name: "Alice".to_string(),
            text: "Hello everyone, welcome!".to_string(),
            is_final: true,
            language: "en-US".to_string(),
            timestamp_ms: 2000,
        };
        if let Some(existing) = captions
            .iter_mut()
            .rev()
            .find(|c| c.participant_id == chunk3.participant_id && !c.is_final)
        {
            existing.text = chunk3.text;
            existing.is_final = chunk3.is_final;
        } else {
            captions.push(chunk3);
        }
        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].text, "Hello everyone, welcome!");
        assert!(captions[0].is_final);

        // 4. Next chunk starts a new entry since previous was final
        let chunk4 = CaptionChunkDto {
            participant_id: id,
            speaker_name: "Alice".to_string(),
            text: "Let's begin.".to_string(),
            is_final: false,
            language: "en-US".to_string(),
            timestamp_ms: 2500,
        };
        if let Some(existing) = captions
            .iter_mut()
            .rev()
            .find(|c| c.participant_id == chunk4.participant_id && !c.is_final)
        {
            existing.text = chunk4.text;
            existing.is_final = chunk4.is_final;
        } else {
            captions.push(chunk4);
        }
        assert_eq!(captions.len(), 2);
        assert_eq!(captions[1].text, "Let's begin.");
    }

    #[test]
    fn test_caption_age_retention_filtering() {
        let now_ms = 100_000u64;
        let max_age_ms = (CAPTION_FADE_DURATION_SECS * 1000.0) as u64; // 6000ms

        let mut list = vec![
            CaptionChunkDto {
                participant_id: Uuid::new_v4(),
                speaker_name: "Old Speaker".to_string(),
                text: "Old message".to_string(),
                is_final: true,
                language: "en-US".to_string(),
                timestamp_ms: now_ms - 15_000, // 15s old -> expired
            },
            CaptionChunkDto {
                participant_id: Uuid::new_v4(),
                speaker_name: "Fresh Speaker".to_string(),
                text: "Fresh message".to_string(),
                is_final: true,
                language: "en-US".to_string(),
                timestamp_ms: now_ms - 2_000, // 2s old -> active
            },
        ];

        list.retain(|cap| {
            if cap.timestamp_ms == 0 || now_ms == 0 {
                true
            } else {
                now_ms.saturating_sub(cap.timestamp_ms) < max_age_ms
            }
        });

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].speaker_name, "Fresh Speaker");
    }
}
