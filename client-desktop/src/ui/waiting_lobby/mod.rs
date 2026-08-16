use crate::app::ConferApp;
use crate::ui::theme::Theme;
use egui::Ui;

mod device_preflight;
mod header_bar;
mod leave_modal;
mod status_card;

pub fn render_waiting_lobby(app: &mut ConferApp, ui: &mut Ui) {
    let full_rect = ui.available_rect_before_wrap();

    // Dark Obsidian canvas background
    ui.painter().rect_filled(full_rect, 0.0, Theme::CANVAS);

    // 1. Top Global Header Bar with Unified Brand Mark & Leave Protection
    header_bar::render_header_bar(app, ui);

    // 2. Centered Pre-Admission Lounge Card
    let available_size = ui.available_size();
    let card_width = 480.0_f32.min(available_size.x - 32.0);

    ui.vertical_centered(|ui| {
        ui.add_space((available_size.y * 0.08).max(16.0));

        Theme::card_frame(ui.style()).show(ui, |ui| {
            ui.set_width(card_width);

            status_card::render_pulse_icon(ui, card_width);

            ui.add_space(8.0);

            status_card::render_room_status(app, ui);

            ui.add_space(10.0);

            status_card::render_offair_banner(ui);

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(12.0);

            status_card::render_identity_strip(app, ui);

            ui.add_space(14.0);

            device_preflight::render_device_preflight(app, ui, card_width);

            ui.add_space(10.0);

            device_preflight::render_mic_vu_meter(app, ui, card_width);
        });
    });

    // 3. Leave Queue Confirmation Modal Dialog
    leave_modal::render_leave_confirmation_modal(app, ui, full_rect);
}
