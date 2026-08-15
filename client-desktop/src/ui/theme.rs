use egui::{Color32, Stroke, Style};

/// Central Design Tokens for the Confer Native Desktop Client.
/// Directly mirrors the official `DESIGN.md` design system specification.
#[allow(dead_code)]
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // -------------------------------------------------------------------------
    // Core Colors
    // -------------------------------------------------------------------------
    pub const CANVAS: Color32 = Color32::from_rgb(11, 12, 14);          // #0B0C0E Obsidian Base
    pub const SURFACE_1: Color32 = Color32::from_rgb(18, 20, 24);       // #121418 Deep Zinc Card
    pub const SURFACE_2: Color32 = Color32::from_rgb(26, 29, 33);       // #1A1D21 Interactive Dark / Pill
    pub const SURFACE_3: Color32 = Color32::from_rgb(38, 42, 48);       // #262A30 Elevated Steel / Hover
    pub const SURFACE_DOCK: Color32 = Color32::from_rgba_premultiplied(18, 20, 24, 250);

    // -------------------------------------------------------------------------
    // Borders & Hairlines
    // -------------------------------------------------------------------------
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(34, 38, 44);   // #22262C Hairline border
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(2, 132, 199);   // #0284C7 Focus / Active border

    // -------------------------------------------------------------------------
    // Semantic Accents
    // -------------------------------------------------------------------------
    pub const PRIMARY: Color32 = Color32::from_rgb(2, 132, 199);         // #0284C7 Confer Blue
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(3, 105, 161);   // #0369A1
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(56, 189, 248);   // #38BDF8 Sky Accent

    pub const EMERALD: Color32 = Color32::from_rgb(16, 185, 129);        // #10B981 Active Speaker / Success
    pub const EMERALD_LIGHT: Color32 = Color32::from_rgb(52, 211, 153);  // #34D399

    pub const CRIMSON: Color32 = Color32::from_rgb(225, 29, 72);         // #E11D48 Destructive / Leave Call
    pub const CRIMSON_HOVER: Color32 = Color32::from_rgb(190, 18, 60);    // #BE123C
    pub const CRIMSON_LIGHT: Color32 = Color32::from_rgb(254, 205, 211); // #FECDD3 Error Banner Text

    pub const AMBER: Color32 = Color32::from_rgb(245, 158, 11);          // #F59E0B Warning / Hand Raised
    pub const AMBER_LIGHT: Color32 = Color32::from_rgb(251, 191, 36);    // #FBBF24

    // -------------------------------------------------------------------------
    // Typography Colors
    // -------------------------------------------------------------------------
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);  // #F8FAFC
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);// #94A3B8
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);    // #64748B
    pub const TEXT_DIM: Color32 = Color32::from_rgb(71, 85, 105);        // #475569

    // -------------------------------------------------------------------------
    // Corner Radii
    // -------------------------------------------------------------------------
    pub const RADIUS_SM: f32 = 6.0;
    pub const RADIUS_MD: f32 = 8.0;
    pub const RADIUS_LG: f32 = 12.0;
    pub const RADIUS_XL: f32 = 16.0;
    pub const RADIUS_PILL: f32 = 24.0;

    // -------------------------------------------------------------------------
    // Pre-Configured Frame Helpers
    // -------------------------------------------------------------------------
    /// Standard card frame for viewfinders, participant profiles, and action cards.
    pub fn card_frame(style: &Style) -> egui::Frame {
        egui::Frame::group(style)
            .fill(Self::SURFACE_1)
            .stroke(Stroke::new(1.0_f32, Self::BORDER_SUBTLE))
            .rounding(Self::RADIUS_XL)
            .inner_margin(20.0)
    }

    /// Elevated focus card frame with primary blue border.
    pub fn focused_card_frame(style: &Style) -> egui::Frame {
        egui::Frame::group(style)
            .fill(Self::SURFACE_1)
            .stroke(Stroke::new(1.0_f32, Self::PRIMARY))
            .rounding(Self::RADIUS_XL)
            .inner_margin(20.0)
    }

    /// Pill frame for status badges, tags, and small indicator chips.
    pub fn pill_frame(style: &Style) -> egui::Frame {
        egui::Frame::group(style)
            .fill(Self::SURFACE_2)
            .stroke(Stroke::new(1.0_f32, Self::BORDER_SUBTLE))
            .rounding(Self::RADIUS_PILL)
            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
    }

    /// Floating bottom control dock capsule frame.
    pub fn dock_frame(style: &Style) -> egui::Frame {
        egui::Frame::group(style)
            .fill(Self::SURFACE_DOCK)
            .stroke(Stroke::new(1.0_f32, Self::SURFACE_3))
            .rounding(Self::RADIUS_PILL)
            .inner_margin(egui::Margin::symmetric(16.0, 7.0))
    }
}
