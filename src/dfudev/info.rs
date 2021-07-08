//! Additional device info based on parsing descriptors

use super::{Device, DfuFunctionalDescriptor, Error, Result, TIMEOUT};

#[derive(Debug)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: String,
    pub manufacturer_string: String,
    pub product_string: String,
    pub serial_number_string: String,
    pub dfu_config_number: u8,
    pub dfu_interface_number: u8,
    pub alt_settings: Vec<(u8, String)>,
    pub dfu_attributes: u8,
    pub dfu_detach_timeout: u16,
    pub dfu_transfer_size: u16,
    pub dfu_version: u16,
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} [0x{:04X?}:0x{:04X?}] v{} {}",
            self.manufacturer_string,
            self.product_string,
            self.vendor_id,
            self.product_id,
            self.device_version,
            self.serial_number_string
        )
    }
}

/// Return additional device information depending on configuration
/// and interface number
pub fn info(
    device: &Device,
    dfu_config_number: u8,
    dfu_interface_number: u8,
) -> Result<DeviceInfo> {
    let handle = device.open()?;
    let language = handle.read_languages(TIMEOUT)?[0];
    let device_desc = device.device_descriptor()?;

    let manufacturer_string = handle
        .read_manufacturer_string(language, &device_desc, TIMEOUT)
        .unwrap_or(String::new());
    let product_string = handle
        .read_product_string(language, &device_desc, TIMEOUT)
        .unwrap_or(String::new());
    let serial_number_string = handle
        .read_serial_number_string(language, &device_desc, TIMEOUT)
        .unwrap_or(String::new());

    let mut alt_settings = Vec::<(u8, String)>::new();

    let mut dfu_attributes = 0;
    let mut dfu_detach_timeout = 0;
    let mut dfu_transfer_size = 0;
    let mut dfu_version = 0;

    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        if config_desc.number() == dfu_config_number {
            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    if interface_desc.interface_number() == dfu_interface_number {
                        let interface_string = match handle.read_interface_string(
                            language,
                            &interface_desc,
                            TIMEOUT,
                        ) {
                            Ok(interface_string) => interface_string,
                            Err(_) => String::from("(unnamed)"),
                        };
                        alt_settings.push((interface_desc.setting_number(), interface_string));

                        // Extra bytes contain the DFU functional descriptor
                        if let Some(extra) = interface_desc.extra() {
                            if extra.len() == 9 && extra[0] == 9 && extra[1] == 0x21 {
                                let func_desc = DfuFunctionalDescriptor::from_bytes(extra);
                                dfu_attributes = func_desc.bmAttributes;
                                dfu_detach_timeout = func_desc.wDetachTimeOut;
                                dfu_transfer_size = func_desc.wTransferSize;
                                dfu_version = func_desc.bcdDFUVersion;
                            } else {
                                return Err(Box::new(Error::NoDfuFunctionalDescriptor));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(DeviceInfo {
        vendor_id: device_desc.vendor_id(),
        product_id: device_desc.product_id(),
        device_version: format!("{}", device_desc.device_version()),
        manufacturer_string,
        product_string,
        serial_number_string,
        dfu_config_number,
        dfu_interface_number,
        alt_settings,
        dfu_attributes,
        dfu_detach_timeout,
        dfu_transfer_size,
        dfu_version,
    })
}
