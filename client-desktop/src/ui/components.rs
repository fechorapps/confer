use egui::{Button, Color32, RichText, Stroke, Ui};
use crate::ui::theme::Theme;

/// Reusable UI Component Primitives for Confer Desktop Client.
#[allow(dead_code)]
pub struct Components;

#[allow(dead_code)]
impl Components {
    /// High-contrast primary action button in Confer Blue.
    pub fn primary_button(text: impl Into<String>, size: f32) -> Button<'static> {
        Button::new(
            RichText::new(text.into())
                .size(size)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(Theme::PRIMARY)
        .rounding(Theme::RADIUS_MD)
    }

    /// Destructive / Leave action button in Crimson Rose.
    pub fn destructive_button(text: impl Into<String>, size: f32) -> Button<'static> {
        Button::new(
            RichText::new(text.into())
                .size(size)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(Theme::CRIMSON)
        .rounding(Theme::RADIUS_MD)
    }

    /// Subtle secondary action button for dialogs and inactive controls.
    pub fn secondary_button(text: impl Into<String>, size: f32) -> Button<'static> {
        Button::new(
            RichText::new(text.into())
                .size(size)
                .color(Theme::TEXT_PRIMARY),
        )
        .fill(Theme::SURFACE_2)
        .rounding(Theme::RADIUS_MD)
    }

    /// Rounded avatar initials circle badge with customizable background color.
    pub fn avatar_badge(ui: &mut Ui, initials: &str, size: f32, font_size: f32, bg_color: Color32) {
        egui::Frame::group(ui.style())
            .fill(bg_color)
            .stroke(Stroke::NONE)
            .rounding(size / 2.0)
            .inner_margin(egui::Margin::symmetric(size * 0.2, size * 0.1))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(initials)
                        .size(font_size)
                        .strong()
                        .color(Color32::WHITE),
                );
            });
    }

    /// Status indicator pill badge (e.g. ● Online, ● 720p HD).
    pub fn status_badge(ui: &mut Ui, text: &str, status_color: Color32) {
        egui::Frame::group(ui.style())
            .fill(Theme::SURFACE_2)
            .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
            .rounding(Theme::RADIUS_PILL)
            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").size(8.0).color(status_color));
                    ui.label(
                        RichText::new(text)
                            .size(11.0)
                            .strong()
                            .color(Theme::TEXT_PRIMARY),
                    );
                });
            });
    }

    /// Error banner container with warning icon.
    pub fn error_banner(ui: &mut Ui, error_msg: &str) {
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(45, 15, 20))
            .stroke(Stroke::new(1.0_f32, Theme::CRIMSON))
            .rounding(Theme::RADIUS_MD)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚠️").size(13.0));
                    ui.label(
                        RichText::new(error_msg)
                            .size(11.5)
                            .color(Theme::CRIMSON_LIGHT),
                    );
                });
            });
    }
}
