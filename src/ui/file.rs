//! UI elements showing file-related information

use eframe::egui;

use crate::{dfudev, DfuFileChecks, Message};

/// Show box with file selection
pub fn selection(
    ui: &mut egui::Ui,
    selected_file: &Option<dfufile::DfuFile>,
    message_sender: &std::sync::mpsc::Sender<Message>,
) {
    let file_path = selected_file.as_ref().map(|file| &file.path);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_space(6.0);
            ui.label("File:");
        });

        ui.group(|ui| {
            ui.set_width(ui.available_width() - 100.0);
            match file_path {
                Some(file_path) => {
                    ui.label(
                        file_path
                            .to_str()
                            .unwrap_or("File path contains invalid characters"),
                    );
                }
                None => {
                    ui.label("");
                }
            }
        });

        let open_button = ui.add(egui::widgets::Button::new("Open...").fill(egui::Color32::BLUE));

        if open_button.clicked() {
            message_sender.send(Message::OpenFileDialog).ok();
        }

        if ui.button("Clear").clicked() {
            message_sender.send(Message::ClearFile).ok();
        }
    });
}

/// Show box with common file information
pub fn common_info(
    ui: &mut egui::Ui,
    dfu_file: &Option<dfufile::DfuFile>,
    dfu_file_checks: &mut DfuFileChecks,
    device_active: bool,
) {
    ui.group(|ui| {
        ui.set_width(ui.available_width() / 12.0 * 4.0);
        ui.set_height(ui.available_height());

        let mut approve_vendor_id = false;
        let mut approve_product_id = false;

        match dfu_file {
            Some(dfu_file) => {
                ui.vertical(|ui| {
                    ui.heading("Metadata");
                    ui.add_space(5.0);
                    egui::Grid::new("file_info").show(ui, |ui| {
                        let vendor_id = dfu_file.suffix.idVendor;
                        let product_id = dfu_file.suffix.idProduct;

                        ui.label("Format:");
                        let text_color = if device_active {
                            if dfu_file_checks.dfu_version_valid {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            }
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };
                        let format_label = ui.add(egui::Label::new(
                            egui::RichText::new(format!("{}", dfu_file.content)).color(text_color),
                        ));
                        if device_active && !dfu_file_checks.dfu_version_valid {
                            format_label
                                .on_hover_text("File format is not appropriate for the device");
                        }
                        ui.end_row();

                        ui.label("Vendor ID:");
                        let text_color = if device_active {
                            if dfu_file_checks.vendor_id_accepted {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            }
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };
                        let vendor_id_label = ui.add(egui::Label::new(
                            egui::RichText::new(format!("0x{:04X}", vendor_id)).color(text_color),
                        ));
                        if device_active && !dfu_file_checks.vendor_id_accepted {
                            vendor_id_label
                                .on_hover_text("Vendor id does not match the one of the device");
                            if ui
                                .button("Approve")
                                .on_hover_text("Accept vendor id for this device")
                                .clicked()
                            {
                                approve_vendor_id = true;
                            }
                        }
                        ui.end_row();

                        ui.label("Product ID:");
                        let text_color = if device_active {
                            if dfu_file_checks.product_id_accepted {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            }
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };
                        let product_id_label = ui.add(egui::Label::new(
                            egui::RichText::new(format!("0x{:04X}", product_id)).color(text_color),
                        ));
                        if device_active && !dfu_file_checks.product_id_accepted {
                            product_id_label
                                .on_hover_text("Product id does not match the one of the device");
                            if ui
                                .button("Approve")
                                .on_hover_text("Accept product id for this device")
                                .clicked()
                            {
                                approve_product_id = true;
                            }
                        }
                        ui.end_row();

                        ui.label("Version:");
                        ui.label(format!("0x{:04X}", dfu_file.suffix.bcdDevice));
                        ui.end_row();

                        ui.label("CRC:");
                        let text_color = if dfu_file_checks.crc_valid {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };
                        let crc_label = ui.add(egui::Label::new(
                            egui::RichText::new(format!("0x{:08X}", dfu_file.suffix.dwCRC))
                                .color(text_color),
                        ));
                        if !dfu_file_checks.crc_valid {
                            crc_label.on_hover_text(
                                "Calculated CRC does not match the value stored in the file",
                            );
                        }
                        ui.end_row();
                    });
                });
            }
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label("No file selected");
                });
            }
        }

        if approve_vendor_id {
            dfu_file_checks.vendor_id_accepted = true;
        }

        if approve_product_id {
            dfu_file_checks.product_id_accepted = true;
        }
    });
}

/// Show box with file content information
pub fn content_info(
    ui: &mut egui::Ui,
    dfu_file: &Option<dfufile::DfuFile>,
    device_info: Option<&dfudev::DeviceInfo>,
) {
    ui.group(|ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());

        match dfu_file {
            Some(dfu_file) => match &dfu_file.content {
                dfufile::Content::Plain => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Plain file. No details available.");
                    });
                }
                dfufile::Content::DfuSe(content) => {
                    ui.vertical(|ui| {
                        ui.heading("Images");
                        ui.add_space(5.0);
                        egui::Grid::new("file_content_info").show(ui, |ui| {
                            ui.label("ID");
                            ui.label("Name");
                            ui.label("Size");
                            ui.label("El.");
                            if device_info.is_some() {
                                ui.label("Target");
                            }
                            ui.end_row();

                            for image in &content.images {
                                ui.label(format!("{}", image.target_prefix.bAlternateSetting));
                                ui.label(match image.target_prefix.bTargetNamed {
                                    0 => "(unnamed)".to_string(),
                                    _ => image.target_prefix.szTargetName.to_string(),
                                });
                                ui.label(format!("{}", image.target_prefix.dwTargetSize));
                                ui.label(format!("{}", image.target_prefix.dwNbElements));
                                if let Some(device_info) = device_info {
                                    let target = device_info.alt_settings.iter().find(|&alt| {
                                        alt.0 == image.target_prefix.bAlternateSetting
                                    });
                                    if let Some(target) = target {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(&target.1)
                                                .color(egui::Color32::GREEN),
                                        ));
                                    } else {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new("Not found")
                                                .color(egui::Color32::RED),
                                        ));
                                    }
                                }
                                ui.end_row();
                            }
                        });
                    });
                }
            },
            None => {
                ui.centered_and_justified(|ui| {
                    ui.label("No file selected");
                });
            }
        }
    });
}
