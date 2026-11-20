pub mod device;
pub mod dialog;
pub mod file;

use egui::{
    self, Align, Align2, CentralPanel, Color32, FontFamily, FontId, Id, LayerId, Layout, MenuBar,
    Order, Panel, Ui, ViewportCommand, global_theme_preference_buttons, menu::MenuButton,
    warn_if_debug_build,
};

use crate::{App, AppEvent, WINDOW_SIZE};

/// Updates the GUI on each frame.
pub fn update(app: &mut App, ui: &mut Ui) {
    let zoom_factor = ui.ctx().zoom_factor();
    if app.zoom_factor != zoom_factor {
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::InnerSize(WINDOW_SIZE * zoom_factor));
        app.zoom_factor = zoom_factor;
    }

    app.device_update_state.device_ready = app.device_id.is_some();
    app.device_update_state.file_ready = app.dfu_file.is_some();
    app.device_update_state.preflight_checks_passed = app.preflight_checks();

    if let Some(modal_dialog) = &mut app.modal_dialog {
        let (should_close, event) = modal_dialog.show(ui.ctx());
        if should_close {
            app.modal_dialog = None;
        }
        if let Some(event) = event {
            app.event_channel.0.send(event).ok();
        }
    }

    // Top panel with menu
    Panel::top("top_panel").show(ui, |ui| {
        ui.add_space(5.0);
        MenuBar::new().ui(ui, |ui| {
            MenuButton::new("File").ui(ui, |ui| {
                if ui.button("Open...").clicked() {
                    app.event_channel.0.send(AppEvent::OpenFileDialog).ok();
                    ui.close();
                }
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                global_theme_preference_buttons(ui);
            });
        });
        ui.add_space(0.1);
    });

    // Bottom panel with app version
    Panel::bottom("bottom_panel").show(ui, |ui| {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            warn_if_debug_build(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.hyperlink_to("Project homepage", env!("CARGO_PKG_HOMEPAGE"));
            });
        });
        ui.add_space(0.5);
    });

    CentralPanel::default().show(ui, |ui| {
        ui.scope(|ui| {
            if app.device_update_state.running {
                ui.disable();
            }

            ui.add_space(5.0);

            device::selection(
                ui,
                &app.devices,
                &app.get_selected_device(),
                &app.event_channel.0,
            );

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.set_height(160.0);

                let device_info = app.get_selected_device().map(|device| &device.info);

                device::common_info(ui, device_info);
                device::memory_info(ui, device_info);
            });

            ui.add_space(5.0);

            file::selection(ui, &app.dfu_file, &app.event_channel.0);

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.set_height(165.0);

                file::common_info(
                    ui,
                    &app.dfu_file,
                    &mut app.dfu_file_checks,
                    app.device_id.is_some(),
                );

                let device_info = app.get_selected_device().map(|device| &device.info);

                file::content_info(ui, &app.dfu_file, device_info);
            });
        });

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.set_height(100.0);
            device::update_controls(ui, &mut app.device_update_state, &app.event_channel.0);
            ui.add_space(10.0);
            let device_info = app.get_selected_device().map(|device| &device.info);
            device::update_progress(ui, &app.device_update_state, device_info);
        });
    });

    // File drag-and-drop
    if !app.device_update_state.running {
        if !ui.ctx().input(|i| i.raw.hovered_files.is_empty()) {
            let painter = ui.ctx().layer_painter(LayerId::new(
                Order::Foreground,
                Id::unique("file_drop_target"),
            ));

            let content_rect = ui.ctx().input(|i| i.content_rect());
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                "Drop DFU file top open.",
                FontId::new(16.0, FontFamily::Proportional),
                ui.ctx().global_style().visuals.warn_fg_color,
            );
        }

        if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
            let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
            if let Some(file) = dropped_files.into_iter().next() {
                app.event_channel
                    .0
                    .send(AppEvent::OpenFile(file.path().to_path_buf()))
                    .ok();
            }
        }
    }
}
