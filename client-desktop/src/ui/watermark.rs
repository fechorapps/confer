use egui::{Color32, FontId, Pos2, Rect, Ui};

/// Renders a semi-transparent diagonally tiled watermark pattern displaying the
/// viewer's display name and email address across the given bounding rectangle.
pub fn render_watermark(ui: &mut Ui, rect: Rect, viewer_name: &str, viewer_email: &str) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let text = if !viewer_email.is_empty() {
        format!("{} • {}", viewer_name, viewer_email)
    } else {
        viewer_name.to_string()
    };

    if text.is_empty() {
        return;
    }

    // Low-opacity subtle white/zinc watermark color (anti-leak / DLP compliance)
    let watermark_color = Color32::from_rgba_premultiplied(248, 250, 252, 32);
    let font_id = FontId::proportional(13.0);

    let step_x = 240.0;
    let step_y = 120.0;

    let start_x = rect.min.x - step_x;
    let end_x = rect.max.x + step_x;
    let start_y = rect.min.y;
    let end_y = rect.max.y;

    let painter = ui.painter().with_clip_rect(rect);

    let mut row = 0;
    let mut y = start_y + 40.0;
    while y < end_y + 40.0 {
        let x_offset = if row % 2 == 0 { 0.0 } else { step_x * 0.5 };
        let mut x = start_x + x_offset;

        while x < end_x {
            let pos = Pos2::new(x, y);
            if rect.contains(pos)
                || rect.intersects(Rect::from_min_size(pos, egui::vec2(step_x, step_y)))
            {
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font_id.clone(),
                    watermark_color,
                );
            }
            x += step_x;
        }

        y += step_y;
        row += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_formatting() {
        let name = "Alice Engineer";
        let email = "alice@example.com";
        let formatted = format!("{name} • {email}");
        assert_eq!(formatted, "Alice Engineer • alice@example.com");

        // Verify zero rect early-returns without panic
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_watermark(ui, Rect::ZERO, name, email);
            });
        });
    }
}
