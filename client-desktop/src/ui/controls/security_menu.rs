use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{RichText, Ui};

pub(super) fn render_security_menu(app: &mut ConferApp, ui: &mut Ui, is_host: bool) {
    // Host Security Policy Menu (Visible to Host)
    if is_host {
        ui.menu_button(
            RichText::new("🛡 Security").size(11.5).strong().color(Theme::PRIMARY_LIGHT),
            |ui| {
                ui.set_min_width(220.0);
                ui.label(RichText::new("Host Governance & DLP").size(11.0).strong().color(Theme::AMBER));
                ui.separator();

                let lock_label = if app.meeting_policy.is_locked {
                    "✓ 🔒 Lock Meeting (Active)"
                } else {
                    "   🔒 Lock Meeting"
                };
                if ui.selectable_label(app.meeting_policy.is_locked, lock_label).clicked() {
                    app.toggle_room_lock();
                }

                let wr_label = if app.meeting_policy.waiting_room_enabled {
                    "✓ ⏳ Enable Waiting Room"
                } else {
                    "   ⏳ Enable Waiting Room"
                };
                if ui.selectable_label(app.meeting_policy.waiting_room_enabled, wr_label).clicked() {
                    app.toggle_waiting_room(!app.meeting_policy.waiting_room_enabled);
                }

                let ss_label = if app.meeting_policy.allow_screen_share {
                    "✓ 🖥 Allow Screen Share"
                } else {
                    "   🖥 Allow Screen Share"
                };
                if ui.selectable_label(app.meeting_policy.allow_screen_share, ss_label).clicked() {
                    app.toggle_allow_screen_share();
                }

                let chat_label = if app.meeting_policy.allow_chat {
                    "✓ 💬 Allow Chat"
                } else {
                    "   💬 Allow Chat"
                };
                if ui.selectable_label(app.meeting_policy.allow_chat, chat_label).clicked() {
                    app.toggle_allow_chat();
                }

                let unmute_label = if app.meeting_policy.allow_unmute {
                    "✓ 🎙 Allow Self Unmute"
                } else {
                    "   🎙 Allow Self Unmute"
                };
                if ui.selectable_label(app.meeting_policy.allow_unmute, unmute_label).clicked() {
                    app.toggle_allow_unmute();
                }

                let wm_label = if app.meeting_policy.watermark_enabled {
                    "✓ 🏷 Visual DLP Watermark"
                } else {
                    "   🏷 Visual DLP Watermark"
                };
                if ui.selectable_label(app.meeting_policy.watermark_enabled, wm_label).clicked() {
                    app.toggle_watermark();
                }
            },
        );
    }
}
