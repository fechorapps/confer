use egui::Color32;

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

#[cfg(test)]
mod tests {
    use super::*;

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
