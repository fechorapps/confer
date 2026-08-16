use egui::{Pos2, Rect, Vec2};

use crate::sdk::protocol::{WhiteboardShapeDto, WhiteboardStrokeDto};

/// Helper function to erase any strokes intersecting with the eraser point
pub fn erase_strokes_at(strokes: &mut Vec<WhiteboardStrokeDto>, point: Pos2, eraser_radius: f32) {
    strokes.retain(|stroke| {
        let threshold = eraser_radius + stroke.stroke_width * 0.5;

        match &stroke.shape {
            WhiteboardShapeDto::Pen { points } => {
                for pair in points.windows(2) {
                    let a = Pos2::new(pair[0][0], pair[0][1]);
                    let b = Pos2::new(pair[1][0], pair[1][1]);
                    if dist_point_to_segment(point, a, b) <= threshold {
                        return false;
                    }
                }
                if let Some(last) = points.last() {
                    let p = Pos2::new(last[0], last[1]);
                    if point.distance(p) <= threshold {
                        return false;
                    }
                }
                true
            }

            WhiteboardShapeDto::Line { start, end } => {
                let a = Pos2::new(start[0], start[1]);
                let b = Pos2::new(end[0], end[1]);
                dist_point_to_segment(point, a, b) > threshold
            }

            WhiteboardShapeDto::Rectangle { start, end } => {
                let p1 = Pos2::new(start[0], start[1]);
                let p2 = Pos2::new(end[0], end[1]);
                let rect = Rect::from_two_pos(p1, p2);

                // Distance to 4 edges
                let top_l = rect.left_top();
                let top_r = rect.right_top();
                let bot_l = rect.left_bottom();
                let bot_r = rect.right_bottom();

                let d1 = dist_point_to_segment(point, top_l, top_r);
                let d2 = dist_point_to_segment(point, top_r, bot_r);
                let d3 = dist_point_to_segment(point, bot_r, bot_l);
                let d4 = dist_point_to_segment(point, bot_l, top_l);

                let min_d = d1.min(d2).min(d3).min(d4);
                min_d > threshold
            }

            WhiteboardShapeDto::Circle { center, radius } => {
                let c = Pos2::new(center[0], center[1]);
                let d = (point.distance(c) - radius).abs();
                d > threshold
            }

            WhiteboardShapeDto::Text {
                pos,
                text,
                font_size,
            } => {
                let p = Pos2::new(pos[0], pos[1]);
                let approx_w = (text.len() as f32 * font_size * 0.6).max(20.0);
                let approx_h = *font_size * 1.3;
                let text_rect =
                    Rect::from_min_size(p, Vec2::new(approx_w, approx_h)).expand(threshold);
                !text_rect.contains(point)
            }
        }
    });
}

/// Calculate the Euclidean distance between a point and a line segment
pub fn dist_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = ab.length_sq();
    if len_sq < 0.0001 {
        return p.distance(a);
    }
    let t = (ap.dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    p.distance(proj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::protocol::WhiteboardColorDto;
    use uuid::Uuid;

    #[test]
    fn test_dist_point_to_segment() {
        let a = Pos2::new(0.0, 0.0);
        let b = Pos2::new(10.0, 0.0);

        // Point directly above segment midpoint
        let p1 = Pos2::new(5.0, 5.0);
        assert!((dist_point_to_segment(p1, a, b) - 5.0).abs() < 0.001);

        // Point beyond end of segment
        let p2 = Pos2::new(14.0, 3.0);
        assert!((dist_point_to_segment(p2, a, b) - 5.0).abs() < 0.001); // distance to (10, 0) is sqrt(4^2 + 3^2) = 5
    }

    #[test]
    fn test_eraser_removes_intersecting_strokes() {
        let mut strokes = vec![
            WhiteboardStrokeDto {
                id: Uuid::new_v4(),
                participant_id: Uuid::new_v4(),
                shape: WhiteboardShapeDto::Line {
                    start: [0.0, 0.0],
                    end: [100.0, 0.0],
                },
                color: WhiteboardColorDto::new(255, 255, 255, 255),
                stroke_width: 2.0,
            },
            WhiteboardStrokeDto {
                id: Uuid::new_v4(),
                participant_id: Uuid::new_v4(),
                shape: WhiteboardShapeDto::Circle {
                    center: [200.0, 200.0],
                    radius: 30.0,
                },
                color: WhiteboardColorDto::new(239, 68, 68, 255),
                stroke_width: 2.0,
            },
            WhiteboardStrokeDto {
                id: Uuid::new_v4(),
                participant_id: Uuid::new_v4(),
                shape: WhiteboardShapeDto::Rectangle {
                    start: [300.0, 300.0],
                    end: [350.0, 350.0],
                },
                color: WhiteboardColorDto::new(34, 197, 94, 255),
                stroke_width: 2.0,
            },
            WhiteboardStrokeDto {
                id: Uuid::new_v4(),
                participant_id: Uuid::new_v4(),
                shape: WhiteboardShapeDto::Pen {
                    points: vec![[400.0, 400.0], [410.0, 410.0], [420.0, 420.0]],
                },
                color: WhiteboardColorDto::new(59, 130, 246, 255),
                stroke_width: 2.0,
            },
            WhiteboardStrokeDto {
                id: Uuid::new_v4(),
                participant_id: Uuid::new_v4(),
                shape: WhiteboardShapeDto::Text {
                    pos: [500.0, 500.0],
                    text: "Diagram 1".to_string(),
                    font_size: 16.0,
                },
                color: WhiteboardColorDto::new(234, 179, 8, 255),
                stroke_width: 2.0,
            },
        ];

        assert_eq!(strokes.len(), 5);

        // Erase over line segment
        erase_strokes_at(&mut strokes, Pos2::new(50.0, 2.0), 10.0);
        assert_eq!(strokes.len(), 4);

        // Erase over circle edge
        erase_strokes_at(&mut strokes, Pos2::new(200.0, 230.0), 10.0);
        assert_eq!(strokes.len(), 3);

        // Erase over rectangle edge
        erase_strokes_at(&mut strokes, Pos2::new(300.0, 320.0), 10.0);
        assert_eq!(strokes.len(), 2);

        // Erase over pen segment
        erase_strokes_at(&mut strokes, Pos2::new(412.0, 412.0), 10.0);
        assert_eq!(strokes.len(), 1);

        // Erase over text bounding box
        erase_strokes_at(&mut strokes, Pos2::new(510.0, 505.0), 10.0);
        assert_eq!(strokes.len(), 0);
    }
}
