use egui::{Color32, RichText, Stroke, Ui};
use crate::app::ConferApp;

pub fn render_controls(app: &mut ConferApp, ui: &mut Ui) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(14.0);

        // Floating Frosted Glass Pill Dock
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgba_premultiplied(18, 20, 23, 240))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 42, 48)))
            .rounding(24.0)
            .inner_margin(egui::Margin::symmetric(18.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Mic Toggle
                    let mic_bg = if app.is_mic_muted { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                    let mic_text = if app.is_mic_muted { "🔇 Unmute" } else { "🎙 Mute" };
                    if ui.add(egui::Button::new(RichText::new(mic_text).size(12.0).color(Color32::WHITE)).fill(mic_bg).rounding(18.0)).clicked() {
                        app.toggle_mic();
                    }

                    // Camera Toggle
                    let cam_bg = if app.is_camera_off { Color32::from_rgb(225, 29, 72) } else { Color32::from_rgb(26, 29, 33) };
                    let cam_text = if app.is_camera_off { "📷 Start Video" } else { "🎥 Stop Video" };
                    if ui.add(egui::Button::new(RichText::new(cam_text).size(12.0).color(Color32::WHITE)).fill(cam_bg).rounding(18.0)).clicked() {
                        app.toggle_camera();
                    }

                    // Screen Share Control (Native Portal or DisplayList Dropdown)
                    if app.is_screen_sharing {
                        if ui.add(egui::Button::new(RichText::new("⏹ Stop Share").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(18.0)).clicked() {
                            app.stop_screen_share();
                        }
                    } else if app.screen_capturer.picker_mode() == crate::media::PickerMode::Native {
                        if ui.add(egui::Button::new(RichText::new("🖥 Share Screen").size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(26, 29, 33)).rounding(18.0)).clicked() {
                            app.start_native_screen_share();
                        }
                    } else {
                        ui.menu_button(RichText::new("🖥 Share").size(12.0).color(Color32::WHITE), |ui| {
                            ui.set_min_width(220.0);
                            ui.label(RichText::new("Select Display to Share").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                            ui.separator();
                            let displays = app.available_displays.clone();
                            for d in displays {
                                if ui.button(RichText::new(&d.label).size(11.0).color(Color32::WHITE)).clicked() {
                                    app.start_screen_share(d);
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    // Hand Raise
                    let hand_bg = if app.is_hand_raised { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(26, 29, 33) };
                    let hand_text = if app.is_hand_raised { "✋ Lower" } else { "✋ Hand" };
                    if ui.add(egui::Button::new(RichText::new(hand_text).size(12.0).color(Color32::WHITE)).fill(hand_bg).rounding(18.0)).clicked() {
                        app.toggle_hand_raise();
                    }

                    // Filters Dropdown
                    ui.menu_button(RichText::new("🎨 Filters").size(12.0).color(Color32::WHITE), |ui| {
                        ui.set_min_width(160.0);
                        ui.label(RichText::new("Video Filters (Meet)").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        ui.separator();
                        for filter in crate::media::filters::VideoFilter::all() {
                            let is_active = app.active_filter == *filter;
                            let label = format!("{}{}", if is_active { "✓ " } else { "   " }, filter.label());
                            if ui.selectable_label(is_active, label).clicked() {
                                app.active_filter = *filter;
                                ui.close_menu();
                            }
                        }
                    });

                    // Backgrounds & Blur Dropdown
                    ui.menu_button(RichText::new("🖼️ Background").size(12.0).color(Color32::WHITE), |ui| {
                        ui.set_min_width(180.0);
                        ui.label(RichText::new("Background & Portrait Blur").size(11.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        ui.separator();
                        for bg in crate::media::background::BackgroundEffect::all() {
                            let is_active = app.active_background == *bg;
                            let label = format!("{}{}", if is_active { "✓ " } else { "   " }, bg.label());
                            if ui.selectable_label(is_active, label).clicked() {
                                app.active_background = bg.clone();
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui.button("📁 Choose Custom Photo...").clicked() {
                            app.choose_custom_background();
                            ui.close_menu();
                        }
                    });

                    // Reactions Dropdown
                    ui.menu_button(RichText::new("✨ React").size(12.0).color(Color32::WHITE), |ui| {
                        ui.horizontal(|ui| {
                            for emoji in ["👍", "❤️", "👏", "🎉", "🚀", "💡", "🔥"] {
                                if ui.add(egui::Button::new(RichText::new(emoji).size(18.0)).fill(Color32::from_rgb(26, 29, 33)).rounding(6.0)).clicked() {
                                    app.send_reaction(emoji);
                                    ui.close_menu();
                                }
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Red Leave Meeting Pill
                    if ui.add(egui::Button::new(RichText::new("✕ Leave").strong().size(12.0).color(Color32::WHITE)).fill(Color32::from_rgb(225, 29, 72)).rounding(18.0)).clicked() {
                        app.leave_meeting();
                    }
                });
            });
    });
}
