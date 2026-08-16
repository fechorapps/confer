use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{RichText, Stroke, Window};

pub fn render_diagnostics(app: &mut ConferApp, ctx: &egui::Context) {
    Window::new("📊 WebRTC Telemetry HUD")
        .open(&mut app.show_diagnostics)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_width(320.0);

            egui::Frame::group(ui.style())
                .fill(Theme::SURFACE_1)
                .stroke(Stroke::new(1.0_f32, Theme::BORDER_SUBTLE))
                .rounding(Theme::RADIUS_MD)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("SYSTEM EFFICIENCY")
                            .size(11.0)
                            .strong()
                            .color(Theme::PRIMARY_LIGHT),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Memory Footprint:")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                        match get_process_memory_mb() {
                            Some(mem_mb) => {
                                let color = if mem_mb < 40.0 {
                                    Theme::EMERALD
                                } else {
                                    Theme::AMBER
                                };
                                ui.label(
                                    RichText::new(format!("{:.1} MB", mem_mb))
                                        .strong()
                                        .size(12.0)
                                        .color(color),
                                );
                                ui.label(
                                    RichText::new("(< 40MB target)")
                                        .size(10.0)
                                        .color(Theme::TEXT_MUTED),
                                );
                            }
                            None => {
                                ui.label(
                                    RichText::new("—")
                                        .strong()
                                        .size(12.0)
                                        .color(Theme::TEXT_MUTED),
                                );
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Latency (RTT):")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                        let rtt = app.rtt_ms;
                        let color = if rtt < 60 {
                            Theme::EMERALD
                        } else {
                            Theme::AMBER
                        };
                        ui.label(
                            RichText::new(format!("{} ms", rtt))
                                .strong()
                                .size(12.0)
                                .color(color),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Packet Loss:")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                        let loss = app.packet_loss_pct;
                        let color = if loss < 0.5 {
                            Theme::EMERALD
                        } else {
                            Theme::CRIMSON
                        };
                        ui.label(
                            RichText::new(format!("{:.1}%", loss))
                                .strong()
                                .size(12.0)
                                .color(color),
                        );
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(
                        RichText::new("MEDIA ENGINE")
                            .size(11.0)
                            .strong()
                            .color(Theme::PRIMARY_LIGHT),
                    );
                    ui.add_space(6.0);

                    // Static capture/encode configuration
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Video Pipeline:")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                        ui.label(
                            RichText::new("V4L2 720p HD (30 FPS)")
                                .size(12.0)
                                .color(Theme::TEXT_PRIMARY),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Audio Encoding:")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                        ui.label(
                            RichText::new("Opus 48kHz Stereo")
                                .size(12.0)
                                .color(Theme::TEXT_PRIMARY),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("AI Noise Filter:")
                                .size(12.0)
                                .color(Theme::TEXT_SECONDARY),
                        );
                        let denoise_text = if app.is_ai_denoise_enabled {
                            "⚡ RNNoise Active"
                        } else {
                            "Off"
                        };
                        let denoise_color = if app.is_ai_denoise_enabled {
                            Theme::EMERALD
                        } else {
                            Theme::TEXT_MUTED
                        };
                        ui.label(
                            RichText::new(denoise_text)
                                .strong()
                                .size(12.0)
                                .color(denoise_color),
                        );
                    });
                });
        });
}

/// Returns the process resident set size in MB, or `None` if it cannot be measured.
fn get_process_memory_mb() -> Option<f32> {
    #[cfg(target_os = "linux")]
    {
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
        let rss_pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
        // SAFETY: sysconf(_SC_PAGESIZE) is always safe to call; it has no side effects.
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if page_size <= 0 {
            return None;
        }
        Some((rss_pages as f64 * page_size as f64 / (1024.0 * 1024.0)) as f32)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
