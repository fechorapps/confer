use egui::{Color32, Stroke, Style};

/// Central Design Tokens for the Confer Native Desktop Client.
/// Directly mirrors the official `DESIGN.md` design system specification.
#[allow(dead_code)]
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // -------------------------------------------------------------------------
    // Core Colors — sourced from Catppuccin Mocha (catppuccin_egui::MOCHA)
    // -------------------------------------------------------------------------
    pub const CANVAS: Color32 = catppuccin_egui::MOCHA.crust; // #11111B
    pub const SURFACE_1: Color32 = catppuccin_egui::MOCHA.mantle; // #181825
    pub const SURFACE_2: Color32 = catppuccin_egui::MOCHA.surface0; // #313244
    pub const SURFACE_3: Color32 = catppuccin_egui::MOCHA.surface1; // #45475A
    pub const SURFACE_DOCK: Color32 = Color32::from_rgba_premultiplied(24, 24, 37, 250); // mantle @ 250 alpha

    // -------------------------------------------------------------------------
    // Borders & Hairlines
    // -------------------------------------------------------------------------
    pub const BORDER_SUBTLE: Color32 = catppuccin_egui::MOCHA.overlay0; // #6C7086
    pub const BORDER_ACTIVE: Color32 = catppuccin_egui::MOCHA.blue; // #89B4FA

    // -------------------------------------------------------------------------
    // Semantic Accents
    // -------------------------------------------------------------------------
    pub const PRIMARY: Color32 = catppuccin_egui::MOCHA.blue; // #89B4FA
                                                              // Catppuccin's palette is flat (no hover variants). Hand-derived by
                                                              // blending PRIMARY 15% toward MOCHA.crust — see spec §1.
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(119, 156, 217);
    pub const PRIMARY_LIGHT: Color32 = catppuccin_egui::MOCHA.sapphire; // #74C7EC

    pub const EMERALD: Color32 = catppuccin_egui::MOCHA.green; // #A6E3A1
    pub const EMERALD_LIGHT: Color32 = catppuccin_egui::MOCHA.teal; // #94E2D5

    pub const CRIMSON: Color32 = catppuccin_egui::MOCHA.red; // #F38BA8
                                                             // Derived the same way as PRIMARY_HOVER — see note above.
    pub const CRIMSON_HOVER: Color32 = Color32::from_rgb(209, 121, 147);
    pub const CRIMSON_LIGHT: Color32 = catppuccin_egui::MOCHA.flamingo; // #F2CDCD

    pub const AMBER: Color32 = catppuccin_egui::MOCHA.peach; // #FAB387
    pub const AMBER_LIGHT: Color32 = catppuccin_egui::MOCHA.yellow; // #F9E2AF

    /// Foreground color for text/icons drawn on top of an accent fill
    /// (EMERALD/PRIMARY/BORDER_ACTIVE/CRIMSON). Catppuccin's accents are
    /// pastel, so a plain white foreground reads as ~1.5-2.3:1 contrast;
    /// dark-on-accent (this token) gives 8-12:1.
    pub const ON_ACCENT: Color32 = catppuccin_egui::MOCHA.crust;

    // -------------------------------------------------------------------------
    // Typography Colors
    // -------------------------------------------------------------------------
    pub const TEXT_PRIMARY: Color32 = catppuccin_egui::MOCHA.text; // #CDD6F4
    pub const TEXT_SECONDARY: Color32 = catppuccin_egui::MOCHA.subtext1; // #BAC2DE
    pub const TEXT_MUTED: Color32 = catppuccin_egui::MOCHA.subtext0; // #A6ADC8
    pub const TEXT_DIM: Color32 = catppuccin_egui::MOCHA.overlay1; // #7F849C

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
            .shadow(egui::epaint::Shadow {
                offset: [0.0, 4.0].into(),
                blur: 16.0,
                spread: 0.0,
                color: Color32::from_rgba_premultiplied(0, 0, 0, 140),
            })
            .inner_margin(egui::Margin::symmetric(16.0, 7.0))
    }
}
