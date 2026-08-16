use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Color32, RichText, Ui};

pub(super) fn render_studio_fx_menu(app: &mut ConferApp, ui: &mut Ui) {
    // Studio FX & Video Tone Settings Menu (⚙)
    ui.menu_button(
        RichText::new("⚙ Studio FX").size(11.5).strong().color(Color32::WHITE),
        |ui| {
            ui.set_min_width(240.0);

            // Audio & AI Denoise
            ui.label(RichText::new("Audio FX & Denoise").size(11.0).strong().color(Theme::PRIMARY_LIGHT));
            let denoise_label = if app.is_ai_denoise_enabled {
                "✓ ⚡ RNNoise 48kHz (Active)"
            } else {
                "   ⚡ RNNoise 48kHz (Off)"
            };
            if ui.selectable_label(app.is_ai_denoise_enabled, denoise_label).clicked() {
                app.toggle_ai_denoise();
            }

            ui.separator();

            // Video Tone Filters
            ui.label(RichText::new("Cinematic Tone Preset").size(11.0).strong().color(Theme::PRIMARY_LIGHT));
            for filter in crate::media::filters::VideoFilter::all() {
                let is_active = app.active_filter == *filter;
                let label = format!("{}{}", if is_active { "✓ " } else { "   " }, filter.label());
                if ui.selectable_label(is_active, label).clicked() {
                    app.active_filter = *filter;
                    ui.close_menu();
                }
            }

            ui.separator();

            // Virtual Backgrounds
            ui.label(RichText::new("Background & Portrait Blur").size(11.0).strong().color(Theme::PRIMARY_LIGHT));
            for mode in crate::media::VirtualBackgroundMode::all() {
                let is_active = app.virtual_bg_mode == *mode;
                let label = format!("{}{}", if is_active { "✓ " } else { "   " }, mode.label());
                if ui.selectable_label(is_active, label).clicked() {
                    app.set_virtual_bg_mode(mode.clone());
                    ui.close_menu();
                }
            }
            if ui.button("📁 Choose Custom Photo...").clicked() {
                app.choose_custom_background();
                ui.close_menu();
            }

            ui.separator();
            ui.label(RichText::new("Keyboard Shortcuts:").size(10.0).color(Theme::TEXT_SECONDARY));
            ui.label(
                RichText::new("Space (Hold): PTT • Ctrl+D: Mic • Ctrl+E: Cam\nCtrl+Shift+S: Screen • Ctrl+Shift+W: Board\nCtrl+Shift+P: Polls • Ctrl+Shift+M: Chat")
                    .size(9.5)
                    .color(Theme::TEXT_MUTED),
            );
        },
    );
}
