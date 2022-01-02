//! UI elements showing device-related information

use crate::{dfudev, DeviceUpdateState, DeviceUpdateStep, Message};
use eframe::egui;

/// Show combobox with devices
pub fn selection(
    ui: &mut egui::Ui,
    devices: &Option<Vec<dfudev::DfuDevice>>,
    selected_device: &Option<&dfudev::DfuDevice>,
    message_sender: &std::sync::mpsc::Sender<Message>,
) {
    let mut device_list = Vec::new();
    let mut device_index = 0;

    if devices.is_some() {
        let selected_device = selected_device;

        for (index, device) in devices.as_ref().unwrap().iter().enumerate() {
            device_list.push(format!(
                "{} | {}",
                &device.info.manufacturer_string, &device.info.product_string
            ));

            if selected_device.is_some() && selected_device.unwrap().id == device.id {
                device_index = index;
            }
        }
    }

    let device_count = device_list.len();

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label("Device:");
        });

        ui.scope(|ui| {
            ui.set_enabled(!device_list.is_empty());

            let combo_box = egui::ComboBox::from_id_source("device_list")
                .width(ui.available_width() - 100.0)
                .show_index(ui, &mut device_index, device_list.len(), |i| {
                    if device_count > 0 {
                        device_list[i].clone()
                    } else {
                        String::from("No devices found")
                    }
                });

            if combo_box.changed() && devices.is_some() {
                for (index, device) in devices.as_ref().unwrap().iter().enumerate() {
                    let d = if devices.is_some() {
                        devices
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find(|&x| x.id == device.id)
                    } else {
                        None
                    };
                    if d.is_some() && index == device_index {
                        message_sender.send(Message::DeviceSelected(device.id)).ok();
                    }
                }
            };
        });

        ui.centered_and_justified(|ui| {
            if ui.button("Rescan").clicked() {
                message_sender.send(Message::RescanDevices).ok();
            };
        });
    });
}

/// Show box with common device information
pub fn common_info(ui: &mut egui::Ui, device_info: Option<&dfudev::DeviceInfo>) {
    ui.group(|ui| {
        ui.set_width(ui.available_width() / 3.0);
        ui.set_height(ui.available_height());

        match device_info {
            Some(device_info) => {
                ui.vertical(|ui| {
                    ui.heading("ID");
                    ui.add_space(5.0);
                    egui::Grid::new("device_info").show(ui, |ui| {
                        ui.label("Vendor ID:");
                        ui.label(format!("0x{:04X}", device_info.vendor_id));
                        ui.end_row();

                        ui.label("Product ID:");
                        ui.label(format!("0x{:04X}", device_info.product_id));
                        ui.end_row();

                        ui.label("Device Version:");
                        ui.label(device_info.device_version.to_owned());
                        ui.end_row();

                        ui.label("Serial No:");
                        ui.label(device_info.serial_number_string.to_owned());
                        ui.end_row();

                        ui.label("DFU Version:");
                        let version_info = if device_info.dfu_version == 0x011A {
                            "(DfuSe)"
                        } else {
                            ""
                        };
                        ui.label(format!(
                            "0x{:04X} {}",
                            device_info.dfu_version, version_info
                        ));
                        ui.end_row();
                    });
                });
            }
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label("No device selected");
                });
            }
        }
    });
}

/// Show box with target information
pub fn memory_info(ui: &mut egui::Ui, device_info: Option<&dfudev::DeviceInfo>) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());

        match device_info {
            Some(device_info) => {
                ui.vertical(|ui| {
                    ui.heading("Memory Segments");

                    ui.add_space(5.0);

                    egui::containers::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("segments_info").show(ui, |ui| {
                            ui.label("ID");
                            ui.label("Name");
                            ui.end_row();

                            for alt_setting in &device_info.alt_settings {
                                ui.label(format!("{}", alt_setting.0));
                                ui.label(alt_setting.1.to_owned());
                                ui.end_row();
                            }
                        });
                    });
                });
            }
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label("No device selected");
                });
            }
        }
    });
}

/// Show update button and additional messages
pub fn update_controls(
    ui: &mut egui::Ui,
    update_state: &mut DeviceUpdateState,
    message_sender: &std::sync::mpsc::Sender<Message>,
) {
    ui.vertical(|ui| {
        ui.set_width(ui.available_width() / 3.0);
        ui.set_height(ui.available_height());

        if update_state.error.is_some() {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.add(egui::Label::new(
                    egui::RichText::new("Error:").color(egui::Color32::RED),
                ));
                ui.add(egui::Label::new(
                    egui::RichText::new(update_state.error.as_ref().unwrap())
                        .color(egui::Color32::RED),
                ));
                ui.add_space(10.0);

                let continue_button =
                    ui.add(egui::widgets::Button::new("Continue").fill(egui::Color32::BLUE));

                if continue_button.clicked() {
                    update_state.error = None;
                };
            });
        } else if update_state.running {
            ui.centered_and_justified(|ui| {
                ui.label("Update in progress...");
            });
        } else if update_state.finished {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.add(egui::Label::new(
                    egui::RichText::new("Update finished successfully.")
                        .color(egui::Color32::GREEN),
                ));
                ui.add_space(10.0);

                let continue_button =
                    ui.add(egui::widgets::Button::new("Continue").fill(egui::Color32::BLUE));

                if continue_button.clicked() {
                    *update_state = DeviceUpdateState::default();
                };
            });
        } else {
            if update_state.device_ready && update_state.file_ready {
                if update_state.preflight_checks_passed {
                    ui.vertical_centered(|ui| {
                        ui.add_space(5.0);
                        ui.add(egui::Label::new(
                            egui::RichText::new("Warning! All data on device will be erased!")
                                .color(egui::Color32::YELLOW),
                        ));
                        ui.add_space(10.0);

                        ui.checkbox(&mut update_state.confirmed, "Confirm to proceed.");

                        ui.add_space(10.0);

                        ui.scope(|ui| {
                            ui.set_enabled(update_state.confirmed);
                            let update_button = ui.add(
                                egui::widgets::Button::new("Start update")
                                    .fill(egui::Color32::BLUE),
                            );

                            if update_button.clicked() {
                                message_sender.send(Message::StartUpdate).ok();
                                update_state.confirmed = false;
                            };
                        });
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(
                                "Some requirements are not met.\nPlease check your settings.",
                            )
                            .color(egui::Color32::RED),
                        ));
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("Please select a device and open a file.")
                            .color(egui::Color32::YELLOW),
                    ));
                });
            }
        }
    });
}

/// Show box with update progress bars
pub fn update_progress(ui: &mut egui::Ui, update_state: &DeviceUpdateState) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        ui.set_enabled(update_state.preflight_checks_passed);

        ui.vertical(|ui| {
            egui::Grid::new("progress_bars")
                .num_columns(2)
                .spacing((20.0, 10.0))
                .show(ui, |ui| {
                    ui.label("Erase");
                    ui.add(
                        egui::ProgressBar::new(update_state.erase_progress)
                            .show_percentage()
                            .animate(
                                update_state
                                    .step
                                    .as_ref()
                                    .map_or(false, |step| *step == DeviceUpdateStep::Erase),
                            ),
                    );
                    ui.end_row();

                    ui.label("Program");
                    ui.add(
                        egui::ProgressBar::new(update_state.program_progress)
                            .show_percentage()
                            .animate(
                                update_state
                                    .step
                                    .as_ref()
                                    .map_or(false, |step| *step == DeviceUpdateStep::Program),
                            ),
                    );
                    ui.end_row();

                    ui.label("Verify");
                    ui.add(
                        egui::ProgressBar::new(update_state.verify_progress)
                            .show_percentage()
                            .animate(
                                update_state
                                    .step
                                    .as_ref()
                                    .map_or(false, |step| *step == DeviceUpdateStep::Verify),
                            ),
                    );
                    ui.end_row();
                })
        });
    });
}
