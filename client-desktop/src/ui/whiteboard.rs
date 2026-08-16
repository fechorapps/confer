use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;
use crate::sdk::protocol::{WhiteboardColorDto, WhiteboardShapeDto, WhiteboardStrokeDto};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteboardTool {
    Pen,
    Line,
    Rectangle,
    Circle,
    Text,
    Eraser,
}

impl WhiteboardTool {
    pub fn label(&self) -> &'static str {
        match self {
            WhiteboardTool::Pen => "✏ Pen",
            WhiteboardTool::Line => "📏 Line",
            WhiteboardTool::Rectangle => "⬛ Rect",
            WhiteboardTool::Circle => "⭕ Circle",
            WhiteboardTool::Text => "🔤 Text",
            WhiteboardTool::Eraser => "🧹 Eraser",
        }
    }

    pub fn all() -> &'static [WhiteboardTool] {
        &[
            WhiteboardTool::Pen,
            WhiteboardTool::Line,
            WhiteboardTool::Rectangle,
            WhiteboardTool::Circle,
            WhiteboardTool::Text,
            WhiteboardTool::Eraser,
        ]
    }
}

pub const WHITEBOARD_COLORS: [(Color32, &str); 6] = [
    (Color32::from_rgb(255, 255, 255), "White"),
    (Color32::from_rgb(239, 68, 68), "Red"),
    (Color32::from_rgb(34, 197, 94), "Green"),
    (Color32::from_rgb(59, 130, 246), "Blue"),
    (Color32::from_rgb(234, 179, 8), "Yellow"),
    (Color32::from_rgb(249, 115, 22), "Orange"),
];

/// Render the interactive whiteboard canvas and toolbar
pub fn render_whiteboard(app: &mut ConferApp, ui: &mut Ui) {
    ui.vertical(|ui| {
        // --- Whiteboard Header / Toolbar ---
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(18, 20, 23))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
            .rounding(8.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    ui.label(
                        RichText::new("🎨 Whiteboard")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(56, 189, 248)),
                    );

                    ui.separator();

                    // --- Tool Selector ---
                    for tool in WhiteboardTool::all() {
                        let is_active = app.whiteboard_tool == *tool;
                        let bg = if is_active {
                            Color32::from_rgb(2, 132, 199)
                        } else {
                            Color32::from_rgb(26, 29, 33)
                        };

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(tool.label()).size(11.0).color(Color32::WHITE),
                                )
                                .fill(bg)
                                .rounding(6.0),
                            )
                            .clicked()
                        {
                            app.whiteboard_tool = *tool;
                            app.whiteboard_current_points.clear();
                            app.whiteboard_drag_start = None;
                            app.whiteboard_drag_current = None;
                        }
                    }

                    ui.separator();

                    // --- Color Swatches ---
                    ui.label(
                        RichText::new("Color:")
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                    for &(color, name) in &WHITEBOARD_COLORS {
                        let is_selected = app.whiteboard_color == color;
                        let stroke = if is_selected {
                            Stroke::new(2.5_f32, Color32::WHITE)
                        } else {
                            Stroke::new(1.0_f32, crate::ui::theme::Theme::BORDER_SUBTLE)
                        };

                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::click());
                        ui.painter().rect(rect, 4.0, color, stroke);
                        if resp.clicked() {
                            app.whiteboard_color = color;
                        }
                        resp.on_hover_text(name);
                    }

                    ui.separator();

                    // --- Stroke Width Slider ---
                    ui.label(
                        RichText::new("Size:")
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                    ui.add(
                        egui::Slider::new(&mut app.whiteboard_stroke_width, 1.0..=20.0)
                            .show_value(true)
                            .text("px"),
                    );

                    ui.separator();

                    // --- Zoom Controls ---
                    ui.label(
                        RichText::new("Zoom:")
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                    if ui
                        .add(
                            egui::Button::new(RichText::new("－").size(11.0).color(Color32::WHITE))
                                .fill(Color32::from_rgb(26, 29, 33))
                                .rounding(4.0),
                        )
                        .on_hover_text("Zoom Out")
                        .clicked()
                    {
                        app.whiteboard_zoom = (app.whiteboard_zoom * 0.85).max(0.25);
                    }

                    ui.label(
                        RichText::new(format!("{:.0}%", app.whiteboard_zoom * 100.0))
                            .size(11.0)
                            .strong()
                            .color(Color32::WHITE),
                    );

                    if ui
                        .add(
                            egui::Button::new(RichText::new("＋").size(11.0).color(Color32::WHITE))
                                .fill(Color32::from_rgb(26, 29, 33))
                                .rounding(4.0),
                        )
                        .on_hover_text("Zoom In")
                        .clicked()
                    {
                        app.whiteboard_zoom = (app.whiteboard_zoom * 1.15).min(4.0);
                    }

                    if app.whiteboard_zoom != 1.0
                        && ui
                            .add(
                                egui::Button::new(
                                    RichText::new("1:1")
                                        .size(10.0)
                                        .color(Color32::from_rgb(148, 163, 184)),
                                )
                                .fill(Color32::from_rgb(26, 29, 33))
                                .rounding(4.0),
                            )
                            .on_hover_text("Reset Zoom (100%)")
                            .clicked()
                    {
                        app.whiteboard_zoom = 1.0;
                    }

                    ui.separator();

                    // --- Undo & Clear ---
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("↩ Undo").size(11.0).color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(38, 42, 48))
                            .rounding(6.0),
                        )
                        .clicked()
                    {
                        app.undo_whiteboard_stroke();
                    }

                    if !app.show_whiteboard_clear_confirmation {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("🗑 Clear").size(11.0).color(Color32::WHITE),
                                )
                                .fill(crate::ui::theme::Theme::CRIMSON)
                                .rounding(crate::ui::theme::Theme::RADIUS_SM),
                            )
                            .on_hover_text("Clear entire whiteboard for all participants")
                            .clicked()
                        {
                            app.show_whiteboard_clear_confirmation = true;
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Clear all?")
                                    .size(11.0)
                                    .strong()
                                    .color(crate::ui::theme::Theme::CRIMSON_LIGHT),
                            );
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Yes, Clear")
                                            .size(10.5)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(crate::ui::theme::Theme::CRIMSON)
                                    .rounding(crate::ui::theme::Theme::RADIUS_SM),
                                )
                                .clicked()
                            {
                                app.clear_whiteboard();
                                app.show_whiteboard_clear_confirmation = false;
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Cancel")
                                            .size(10.5)
                                            .color(crate::ui::theme::Theme::TEXT_PRIMARY),
                                    )
                                    .fill(crate::ui::theme::Theme::SURFACE_3)
                                    .rounding(crate::ui::theme::Theme::RADIUS_SM),
                                )
                                .clicked()
                            {
                                app.show_whiteboard_clear_confirmation = false;
                            }
                        });
                    }

                    // --- Close Whiteboard Button ---
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("✕ Close Stage")
                                        .size(11.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(26, 29, 33))
                                .rounding(6.0),
                            )
                            .clicked()
                        {
                            app.is_whiteboard_active = false;
                        }
                    });
                });

                // Text tool input bar when Text tool is active
                if app.whiteboard_tool == WhiteboardTool::Text {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Click on canvas to set pos, or enter text:")
                                .size(11.0)
                                .color(Color32::from_rgb(56, 189, 248)),
                        );
                        let text_edit = ui.add(
                            egui::TextEdit::singleline(&mut app.whiteboard_text_input)
                                .desired_width(200.0)
                                .hint_text("Type whiteboard text..."),
                        );

                        if (ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Place Text").size(11.0).color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(2, 132, 199))
                                .rounding(4.0),
                            )
                            .clicked()
                            || (text_edit.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                            && !app.whiteboard_text_input.trim().is_empty()
                        {
                            let pos = app.whiteboard_text_pos.unwrap_or(Pos2::new(100.0, 100.0));
                            app.commit_whiteboard_text(pos);
                        }
                    });
                }
            });

        ui.add_space(6.0);

        // --- Main Whiteboard Drawing Canvas ---
        let canvas_size = ui.available_size() - Vec2::new(0.0, 10.0);
        let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::click_and_drag());
        let canvas_rect = response.rect;

        // Clip painter to canvas bounds
        let painter = painter.with_clip_rect(canvas_rect);

        // Canvas background: Dark slate obsidian with subtle grid
        painter.rect_filled(canvas_rect, 8.0, Color32::from_rgb(15, 17, 21));
        painter.rect_stroke(
            canvas_rect,
            8.0,
            Stroke::new(1.5_f32, Color32::from_rgb(38, 42, 48)),
        );

        // Subtle background dot grid for precision
        let grid_spacing = 32.0;
        let mut x = canvas_rect.min.x + grid_spacing;
        while x < canvas_rect.max.x {
            let mut y = canvas_rect.min.y + grid_spacing;
            while y < canvas_rect.max.y {
                painter.circle_filled(
                    Pos2::new(x, y),
                    1.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 12),
                );
                y += grid_spacing;
            }
            x += grid_spacing;
        }

        let zoom = app.whiteboard_zoom.clamp(0.25, 4.0);

        // Interactive mouse wheel zoom on hover
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta);
            if scroll.y != 0.0 {
                let factor = if scroll.y > 0.0 { 1.08 } else { 0.92 };
                app.whiteboard_zoom = (app.whiteboard_zoom * factor).clamp(0.25, 4.0);
            }
        }

        // Middle-click or right-click canvas panning
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            app.whiteboard_pan += response.drag_delta();
        }

        let pan = app.whiteboard_pan;
        let to_screen = move |x: f32, y: f32| -> Pos2 {
            canvas_rect.min + pan + Vec2::new(x * zoom, y * zoom)
        };

        // --- Handle User Interactions on Canvas ---
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if canvas_rect.contains(pointer_pos) {
                let rel_pos = pointer_pos - canvas_rect.min - app.whiteboard_pan;
                let rel_pos2 = Pos2::new(rel_pos.x / zoom, rel_pos.y / zoom);

                match app.whiteboard_tool {
                    WhiteboardTool::Pen => {
                        if response.drag_started() {
                            app.whiteboard_current_points = vec![rel_pos2];
                        } else if response.dragged() {
                            if let Some(&last) = app.whiteboard_current_points.last() {
                                if last.distance(rel_pos2) > 2.0 {
                                    app.whiteboard_current_points.push(rel_pos2);
                                }
                            } else {
                                app.whiteboard_current_points.push(rel_pos2);
                            }
                        } else if response.drag_stopped()
                            && !app.whiteboard_current_points.is_empty()
                        {
                            let points = if app.whiteboard_current_points.len() == 1 {
                                vec![
                                    [
                                        app.whiteboard_current_points[0].x,
                                        app.whiteboard_current_points[0].y,
                                    ],
                                    [
                                        app.whiteboard_current_points[0].x + 0.1,
                                        app.whiteboard_current_points[0].y + 0.1,
                                    ],
                                ]
                            } else {
                                app.whiteboard_current_points
                                    .iter()
                                    .map(|p| [p.x, p.y])
                                    .collect()
                            };

                            let stroke = WhiteboardStrokeDto {
                                id: Uuid::new_v4(),
                                participant_id: app.my_participant_id.unwrap_or(Uuid::nil()),
                                shape: WhiteboardShapeDto::Pen { points },
                                color: WhiteboardColorDto::new(
                                    app.whiteboard_color.r(),
                                    app.whiteboard_color.g(),
                                    app.whiteboard_color.b(),
                                    app.whiteboard_color.a(),
                                ),
                                stroke_width: app.whiteboard_stroke_width,
                            };
                            app.add_whiteboard_stroke(stroke);
                            app.whiteboard_current_points.clear();
                        }
                    }

                    WhiteboardTool::Line => {
                        if response.drag_started() {
                            app.whiteboard_drag_start = Some(rel_pos2);
                            app.whiteboard_drag_current = Some(rel_pos2);
                        } else if response.dragged() {
                            app.whiteboard_drag_current = Some(rel_pos2);
                        } else if response.drag_stopped() {
                            if let (Some(start), Some(end)) =
                                (app.whiteboard_drag_start, app.whiteboard_drag_current)
                            {
                                if (start.x - end.x).abs() > 2.0 && (start.y - end.y).abs() > 2.0 {
                                    let stroke = WhiteboardStrokeDto {
                                        id: Uuid::new_v4(),
                                        participant_id: app
                                            .my_participant_id
                                            .unwrap_or(Uuid::nil()),
                                        shape: WhiteboardShapeDto::Line {
                                            start: [start.x, start.y],
                                            end: [end.x, end.y],
                                        },
                                        color: WhiteboardColorDto::new(
                                            app.whiteboard_color.r(),
                                            app.whiteboard_color.g(),
                                            app.whiteboard_color.b(),
                                            app.whiteboard_color.a(),
                                        ),
                                        stroke_width: app.whiteboard_stroke_width,
                                    };
                                    app.add_whiteboard_stroke(stroke);
                                }
                            }
                            app.whiteboard_drag_start = None;
                            app.whiteboard_drag_current = None;
                        }
                    }

                    WhiteboardTool::Rectangle => {
                        if response.drag_started() {
                            app.whiteboard_drag_start = Some(rel_pos2);
                            app.whiteboard_drag_current = Some(rel_pos2);
                        } else if response.dragged() {
                            app.whiteboard_drag_current = Some(rel_pos2);
                        } else if response.drag_stopped() {
                            if let (Some(start), Some(end)) =
                                (app.whiteboard_drag_start, app.whiteboard_drag_current)
                            {
                                if start.distance(end) > 2.0 {
                                    let stroke = WhiteboardStrokeDto {
                                        id: Uuid::new_v4(),
                                        participant_id: app
                                            .my_participant_id
                                            .unwrap_or(Uuid::nil()),
                                        shape: WhiteboardShapeDto::Rectangle {
                                            start: [start.x, start.y],
                                            end: [end.x, end.y],
                                        },
                                        color: WhiteboardColorDto::new(
                                            app.whiteboard_color.r(),
                                            app.whiteboard_color.g(),
                                            app.whiteboard_color.b(),
                                            app.whiteboard_color.a(),
                                        ),
                                        stroke_width: app.whiteboard_stroke_width,
                                    };
                                    app.add_whiteboard_stroke(stroke);
                                }
                            }
                            app.whiteboard_drag_start = None;
                            app.whiteboard_drag_current = None;
                        }
                    }

                    WhiteboardTool::Circle => {
                        if response.drag_started() {
                            app.whiteboard_drag_start = Some(rel_pos2);
                            app.whiteboard_drag_current = Some(rel_pos2);
                        } else if response.dragged() {
                            app.whiteboard_drag_current = Some(rel_pos2);
                        } else if response.drag_stopped() {
                            if let (Some(start), Some(end)) =
                                (app.whiteboard_drag_start, app.whiteboard_drag_current)
                            {
                                let radius = start.distance(end);
                                if radius > 2.0 {
                                    let stroke = WhiteboardStrokeDto {
                                        id: Uuid::new_v4(),
                                        participant_id: app
                                            .my_participant_id
                                            .unwrap_or(Uuid::nil()),
                                        shape: WhiteboardShapeDto::Circle {
                                            center: [start.x, start.y],
                                            radius,
                                        },
                                        color: WhiteboardColorDto::new(
                                            app.whiteboard_color.r(),
                                            app.whiteboard_color.g(),
                                            app.whiteboard_color.b(),
                                            app.whiteboard_color.a(),
                                        ),
                                        stroke_width: app.whiteboard_stroke_width,
                                    };
                                    app.add_whiteboard_stroke(stroke);
                                }
                            }
                            app.whiteboard_drag_start = None;
                            app.whiteboard_drag_current = None;
                        }
                    }

                    WhiteboardTool::Text => {
                        if response.clicked() {
                            app.whiteboard_text_pos = Some(rel_pos2);
                        }
                    }

                    WhiteboardTool::Eraser => {
                        if response.dragged() || response.clicked() {
                            erase_strokes_at(&mut app.whiteboard_strokes, rel_pos2, 16.0);
                        }
                    }
                }
            }
        }

        // --- Render All Committed Whiteboard Strokes ---
        for stroke in &app.whiteboard_strokes {
            let color = Color32::from_rgba_unmultiplied(
                stroke.color.r,
                stroke.color.g,
                stroke.color.b,
                stroke.color.a,
            );
            let scaled_width = (stroke.stroke_width * zoom).max(1.0);
            let egui_stroke = Stroke::new(scaled_width, color);

            match &stroke.shape {
                WhiteboardShapeDto::Pen { points } => {
                    for pair in points.windows(2) {
                        let p1 = to_screen(pair[0][0], pair[0][1]);
                        let p2 = to_screen(pair[1][0], pair[1][1]);
                        painter.line_segment([p1, p2], egui_stroke);
                        painter.circle_filled(p1, scaled_width * 0.5, color);
                    }
                    if let Some(last) = points.last() {
                        let p = to_screen(last[0], last[1]);
                        painter.circle_filled(p, scaled_width * 0.5, color);
                    }
                }

                WhiteboardShapeDto::Line { start, end } => {
                    let p1 = to_screen(start[0], start[1]);
                    let p2 = to_screen(end[0], end[1]);
                    painter.line_segment([p1, p2], egui_stroke);
                }

                WhiteboardShapeDto::Rectangle { start, end } => {
                    let p1 = to_screen(start[0], start[1]);
                    let p2 = to_screen(end[0], end[1]);
                    painter.rect_stroke(Rect::from_two_pos(p1, p2), 2.0, egui_stroke);
                }

                WhiteboardShapeDto::Circle { center, radius } => {
                    let c = to_screen(center[0], center[1]);
                    painter.circle_stroke(c, *radius * zoom, egui_stroke);
                }

                WhiteboardShapeDto::Text {
                    pos,
                    text,
                    font_size,
                } => {
                    let p = to_screen(pos[0], pos[1]);
                    painter.text(
                        p,
                        Align2::LEFT_TOP,
                        text,
                        FontId::proportional((*font_size * zoom).max(8.0)),
                        color,
                    );
                }
            }
        }

        // --- Render In-Progress Drawing Preview ---
        let current_color = app.whiteboard_color;
        let preview_width = (app.whiteboard_stroke_width * zoom).max(1.0);
        let current_stroke = Stroke::new(preview_width, current_color);

        match app.whiteboard_tool {
            WhiteboardTool::Pen => {
                let pts = &app.whiteboard_current_points;
                for pair in pts.windows(2) {
                    let p1 = to_screen(pair[0].x, pair[0].y);
                    let p2 = to_screen(pair[1].x, pair[1].y);
                    painter.line_segment([p1, p2], current_stroke);
                    painter.circle_filled(p1, preview_width * 0.5, current_color);
                }
            }

            WhiteboardTool::Line => {
                if let (Some(start), Some(end)) =
                    (app.whiteboard_drag_start, app.whiteboard_drag_current)
                {
                    let p1 = to_screen(start.x, start.y);
                    let p2 = to_screen(end.x, end.y);
                    painter.line_segment([p1, p2], current_stroke);
                }
            }

            WhiteboardTool::Rectangle => {
                if let (Some(start), Some(end)) =
                    (app.whiteboard_drag_start, app.whiteboard_drag_current)
                {
                    let p1 = to_screen(start.x, start.y);
                    let p2 = to_screen(end.x, end.y);
                    painter.rect_stroke(Rect::from_two_pos(p1, p2), 2.0, current_stroke);
                }
            }

            WhiteboardTool::Circle => {
                if let (Some(start), Some(end)) =
                    (app.whiteboard_drag_start, app.whiteboard_drag_current)
                {
                    let radius = start.distance(end) * zoom;
                    let c = to_screen(start.x, start.y);
                    painter.circle_stroke(c, radius, current_stroke);
                }
            }

            WhiteboardTool::Text => {
                if let Some(pos) = app.whiteboard_text_pos {
                    let p = to_screen(pos.x, pos.y);
                    painter.circle_filled(p, 4.0, Color32::from_rgb(56, 189, 248));
                    painter.text(
                        p + Vec2::new(6.0, -8.0),
                        Align2::LEFT_TOP,
                        "Type in toolbar above",
                        FontId::proportional(11.0),
                        Color32::from_rgb(148, 163, 184),
                    );
                }
            }

            WhiteboardTool::Eraser => {
                if let Some(pos) = response.hover_pos() {
                    if canvas_rect.contains(pos) {
                        painter.circle_stroke(
                            pos,
                            16.0,
                            Stroke::new(1.5_f32, Color32::from_rgb(225, 29, 72)),
                        );
                    }
                }
            }
        }
    });
}

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

    #[test]
    fn test_whiteboard_tools_all() {
        let tools = WhiteboardTool::all();
        assert_eq!(tools.len(), 6);
        assert_eq!(WhiteboardTool::Pen.label(), "✏ Pen");
        assert_eq!(WhiteboardTool::Line.label(), "📏 Line");
        assert_eq!(WhiteboardTool::Rectangle.label(), "⬛ Rect");
        assert_eq!(WhiteboardTool::Circle.label(), "⭕ Circle");
        assert_eq!(WhiteboardTool::Text.label(), "🔤 Text");
        assert_eq!(WhiteboardTool::Eraser.label(), "🧹 Eraser");
    }

    #[test]
    fn test_whiteboard_color_palette() {
        assert_eq!(WHITEBOARD_COLORS.len(), 6);
        assert_eq!(WHITEBOARD_COLORS[0].1, "White");
        assert_eq!(WHITEBOARD_COLORS[1].1, "Red");
        assert_eq!(WHITEBOARD_COLORS[2].1, "Green");
        assert_eq!(WHITEBOARD_COLORS[3].1, "Blue");
        assert_eq!(WHITEBOARD_COLORS[4].1, "Yellow");
        assert_eq!(WHITEBOARD_COLORS[5].1, "Orange");
    }
}
