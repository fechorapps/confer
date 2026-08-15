use crate::app::ConferApp;
use egui::{Color32, RichText, Stroke, Window};

pub fn render_diagnostics(app: &mut ConferApp, ctx: &egui::Context) {
    Window::new("📊 WebRTC Telemetry HUD")
        .open(&mut app.show_diagnostics)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_width(320.0);

            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(18, 20, 23))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(34, 38, 44)))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("SYSTEM EFFICIENCY")
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(56, 189, 248)),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Memory Footprint:")
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        match get_process_memory_mb() {
                            Some(mem_mb) => {
                                let color = if mem_mb < 40.0 {
                                    Color32::from_rgb(16, 185, 129)
                                } else {
                                    Color32::from_rgb(245, 158, 11)
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
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            }
                            None => {
                                ui.label(
                                    RichText::new("—")
                                        .strong()
                                        .size(12.0)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Latency (RTT):")
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        let rtt = app.rtt_ms;
                        let color = if rtt < 60 {
                            Color32::from_rgb(16, 185, 129)
                        } else {
                            Color32::from_rgb(245, 158, 11)
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
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        let loss = app.packet_loss_pct;
                        let color = if loss < 0.5 {
                            Color32::from_rgb(16, 185, 129)
                        } else {
                            Color32::from_rgb(244, 63, 94)
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
                            .color(Color32::from_rgb(56, 189, 248)),
                    );
                    ui.add_space(6.0);

                    // Static capture/encode configuration (not live telemetry)
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Video Pipeline:")
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            RichText::new("V4L2 720p HD (30 FPS)")
                                .size(12.0)
                                .color(Color32::from_rgb(248, 250, 252)),
                        );
                        ui.label(
                            RichText::new("(config)")
                                .size(10.0)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });

                    // Simulcast layer selection is not tracked client-side yet
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Simulcast Layer:")
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            RichText::new("—")
                                .size(12.0)
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Audio Encoding:")
                                .size(12.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(
                            RichText::new("Opus 48kHz Stereo")
                                .size(12.0)
                                .color(Color32::from_rgb(248, 250, 252)),
                        );
                        ui.label(
                            RichText::new("(config)")
                                .size(10.0)
                                .color(Color32::from_rgb(100, 116, 139)),
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
