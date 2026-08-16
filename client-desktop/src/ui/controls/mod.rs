use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::{Pos2, Stroke, Ui};

mod engagement_cluster;
mod leave_button;
mod media_cluster;
mod security_menu;
mod side_panels;
mod studio_fx_menu;

use engagement_cluster::render_engagement_cluster;
use leave_button::render_leave_button;
use media_cluster::render_media_cluster;
use security_menu::render_security_menu;
use side_panels::render_side_panel_buttons;
use studio_fx_menu::render_studio_fx_menu;

pub fn render_controls(app: &mut ConferApp, ui: &mut Ui) {
    let is_host = app.my_role == "host";
    let has_side_drawer = app.show_chat || app.show_roster || app.show_polls;
    let is_compact = ui.available_width() < 1060.0 || has_side_drawer;

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(14.0);

        // Floating Frosted Glass Capsule Dock
        Theme::dock_frame(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // =========================================================
                // CLUSTER 1: CORE HARDWARE MEDIA (MIC, CAM, SCREEN SHARE)
                // =========================================================
                render_media_cluster(app, ui);

                // Precision Bounded Divider 1
                render_divider(ui);

                // =========================================================
                // CLUSTER 2: COLLABORATION & AUDIENCE ENGAGEMENT
                // =========================================================
                render_engagement_cluster(app, ui, is_compact);

                // Precision Bounded Divider 2
                render_divider(ui);

                // =========================================================
                // CLUSTER 3: SIDE PANELS, STUDIO FX & HOST GOVERNANCE
                // =========================================================
                render_side_panel_buttons(app, ui);
                render_studio_fx_menu(app, ui);
                render_security_menu(app, ui, is_host);

                // Precision Bounded Divider 3
                render_divider(ui);

                // =========================================================
                // CLUSTER 4: HIGH-STAKES LEAVE ACTION (CONFIRMATION GUARD)
                // =========================================================
                render_leave_button(app, ui);
            });
        });
    });
}

/// Precision bounded divider: a short vertical rule painted at the current
/// cursor position, used to visually separate control clusters in the dock.
fn render_divider(ui: &mut Ui) {
    ui.add_space(4.0);
    let div_x = ui.cursor().min.x;
    let div_cy = ui.available_rect_before_wrap().center().y;
    ui.painter().line_segment(
        [Pos2::new(div_x, div_cy - 9.0), Pos2::new(div_x, div_cy + 9.0)],
        Stroke::new(1.0_f32, Theme::BORDER_SUBTLE),
    );
    ui.add_space(4.0);
}
