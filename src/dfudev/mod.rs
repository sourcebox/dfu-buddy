//! USB DFU device management
//!
//! Reference: [DFU 1.1 Specification](https://www.usb.org/sites/default/files/DFU_1.1.pdf)

#![allow(dead_code)]

pub mod dfuse;
pub mod info;
pub mod states;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use anyhow::{anyhow, Result};
pub use rusb::has_hotplug;
use rusb::{constants, GlobalContext};

pub use info::DeviceInfo;
pub use states::{DeviceStateCode, DeviceStatusCode};

pub type Device = rusb::Device<GlobalContext>;

////////////////////////////////////////////////////////////////////////////////

/// Device Firmware Upgrade Code
const INTERFACE_SUBCLASS_DFU: u8 = 0x01;

/// Device timeout
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Number of retries when polling status
const NUM_POLLING_RETRIES: usize = 5;

/// Requests module, each constant is a tuple of (request_type, request)
mod requests {
    /// Generate a detach-attach sequence on the bus
    pub const DFU_DETACH: (u8, u8) = (0b00100001, 0);

    /// Download firmware data from host to device
    pub const DFU_DNLOAD: (u8, u8) = (0b00100001, 1);

    /// Upload firmware data from device to host
    pub const DFU_UPLOAD: (u8, u8) = (0b10100001, 2);

    /// Request the status from the device
    pub const DFU_GETSTATUS: (u8, u8) = (0b10100001, 3);

    /// Clear device error status
    pub const DFU_CLRSTATUS: (u8, u8) = (0b00100001, 4);

    /// Request a report about the state of the device
    pub const DFU_GETSTATE: (u8, u8) = (0b10100001, 5);

    /// Abort operations and return to the idle state
    pub const DFU_ABORT: (u8, u8) = (0b00100001, 6);
}

////////////////////////////////////////////////////////////////////////////////

pub struct DfuDevice {
    /// Unique hash based on vendor id, product id and serial
    pub id: u64,

    /// Additional info containing strings and alt settings
    pub info: DeviceInfo,

    /// Instance of rusb::Device
    dev: Device,

    /// rusb device handle
    handle: Option<rusb::DeviceHandle<rusb::GlobalContext>>,
}

impl Hash for DfuDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.info.vendor_id.hash(state);
        self.info.product_id.hash(state);
        self.info.serial_number_string.hash(state);
    }
}

impl DfuDevice {
    /// Return a vector of all devices with DFU capability
    ///
    /// - If `include_runtime` is set to `false`, only devices in DFU mode are returned
    /// - If `include_runtime` is set to `true`, also devices in runtime configuration
    ///   are returned
    pub fn find(include_runtime: bool) -> Result<Option<Vec<Self>>> {
        let mut devices = Vec::new();

        for device in rusb::devices()?.iter() {
            let mut found = false;
            let mut config_number: u8 = 0;
            let mut interface_number: u8 = 0;

            let device_desc = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => continue,
            };

            'outer: for n in 0..device_desc.num_configurations() {
                let config_desc = match device.config_descriptor(n) {
                    Ok(desc) => desc,
                    Err(_) => continue,
                };

                for interface in config_desc.interfaces() {
                    for interface_desc in interface.descriptors() {
                        if interface_desc.class_code() == constants::LIBUSB_CLASS_APPLICATION
                            && interface_desc.sub_class_code() == INTERFACE_SUBCLASS_DFU
                            && (interface_desc.interface_number() == 0 || include_runtime)
                        {
                            found = true;
                            config_number = config_desc.number();
                            interface_number = interface_desc.interface_number();
                            break 'outer;
                        }
                    }
                }
            }

            if found {
                let info = info::info(&device, config_number, interface_number)?;
                let mut device = Self {
                    id: 0,
                    dev: device,
                    info,
                    handle: None,
                };
                let mut hasher = DefaultHasher::new();
                device.hash(&mut hasher);
                let hash = hasher.finish();
                device.id = hash;
                devices.push(device);
            }
        }

        let result = if !devices.is_empty() {
            Some(devices)
        } else {
            None
        };

        Ok(result)
    }

    /// Find a device by its id
    pub fn find_by_id(id: u64) -> Result<Option<Self>> {
        let devices = Self::find(false)?;

        if let Some(devices) = devices {
            Ok(devices.into_iter().find(|x| x.id == id))
        } else {
            Ok(None)
        }
    }

    /// Open the device
    pub fn open(&mut self) -> Result<()> {
        self.handle = Some(self.dev.open()?);

        Ok(())
    }

    /// Close the device
    pub fn close(&mut self) {
        self.handle = None;
    }

    /// Return the device handle as result
    pub fn handle(&self) -> Result<&rusb::DeviceHandle<rusb::GlobalContext>> {
        self.handle.as_ref().ok_or(anyhow!(Error::NoDeviceHandle))
    }

    /// Send a DFU_DETACH request
    pub fn detach_request(&self) -> Result<()> {
        self.handle()?.write_control(
            requests::DFU_DETACH.0,
            requests::DFU_DETACH.1,
            0,
            0,
            &[],
            TIMEOUT,
        )?;

        Ok(())
    }

    /// Send a DFU_DNLOAD request
    ///
    /// A buffer containing data is written to the device and the number
    /// of transferred bytes is returned
    pub fn download_request(&self, block_num: u16, data: &[u8]) -> Result<usize> {
        let transfer_size = self.handle()?.write_control(
            requests::DFU_DNLOAD.0,
            requests::DFU_DNLOAD.1,
            block_num,
            0,
            data,
            TIMEOUT,
        )?;

        Ok(transfer_size)
    }

    /// Send a DFU_UPLOAD request
    ///
    /// A buffer is filled with data from the device and the number
    /// of transferred bytes is returned
    pub fn upload_request(&self, block_num: u16, data: &mut [u8]) -> Result<usize> {
        let transfer_size = self.handle()?.read_control(
            requests::DFU_UPLOAD.0,
            requests::DFU_UPLOAD.1,
            block_num,
            0,
            data,
            TIMEOUT,
        )?;

        Ok(transfer_size)
    }

    /// Send a DFU_GETSTATUS request
    ///
    /// A `DeviceStatus` struct is returned containing the response
    /// in a convenient format
    pub fn getstatus_request(&self) -> Result<DeviceStatusResponse> {
        let mut buffer = [0; 6];

        self.handle()?.read_control(
            requests::DFU_GETSTATUS.0,
            requests::DFU_GETSTATUS.1,
            0,
            0,
            &mut buffer,
            TIMEOUT,
        )?;

        Ok(DeviceStatusResponse::from_bytes(&buffer))
    }

    /// Send a DFU_CLRSTATUS request
    pub fn clrstatus_request(&self) -> Result<()> {
        self.handle()?.write_control(
            requests::DFU_CLRSTATUS.0,
            requests::DFU_CLRSTATUS.1,
            0,
            0,
            &[],
            TIMEOUT,
        )?;

        Ok(())
    }

    /// Send a DFU_GETSTATE request
    pub fn getstate_request(&self) -> Result<u8> {
        let mut buffer = [0; 1];

        self.handle()?.read_control(
            requests::DFU_GETSTATE.0,
            requests::DFU_GETSTATE.1,
            0,
            0,
            &mut buffer,
            TIMEOUT,
        )?;

        Ok(buffer[0])
    }

    /// Send a DFU_ABORT request
    pub fn abort_request(&self) -> Result<()> {
        self.handle()?.write_control(
            requests::DFU_ABORT.0,
            requests::DFU_ABORT.1,
            0,
            0,
            &[],
            TIMEOUT,
        )?;

        Ok(())
    }

    pub fn wait_for_status_response(&self, timeout: u64) -> Result<DeviceStatusResponse> {
        let mut retries = 0;

        loop {
            // Wait the time requested by the device in status response
            std::thread::sleep(std::time::Duration::from_millis(timeout));

            // Status response must have state dfuDNLOAD_IDLE
            let status = self.getstatus_request();
            if let Ok(status) = status {
                if status.bState != states::DeviceStateCode::dfuDNLOAD_IDLE {
                    return Err(anyhow!(Error::InvalidDeviceState(status.bState)));
                }
                return Ok(status);
            } else {
                // This happens if device reports a too short bwPollTimeout
                // Retry a few times to get around this issue
                if retries > NUM_POLLING_RETRIES {
                    return Err(anyhow!(Error::TooManyGetStatusRetries));
                }
                retries += 1;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// DFU functional descriptor, see DFU 1.1 specification table 4.2
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct DfuFunctionalDescriptor {
    /// Size of this descriptor, in bytes.
    bLength: u8,

    /// DFU FUNCTIONAL descriptor type
    bDescriptorType: u8,

    /// DFU attributes
    bmAttributes: u8,

    /// Time, in milliseconds, that the device will wait after receipt of the
    /// DFU_DETACH request.
    wDetachTimeOut: u16,

    /// Maximum number of bytes that the device can accept per
    /// control-write transaction.
    wTransferSize: u16,

    /// Numeric expression identifying the version of the DFU
    /// specification release.
    bcdDFUVersion: u16,
}

impl DfuFunctionalDescriptor {
    /// Creates a new descriptor from a buffer of u8 values
    pub fn from_bytes(buffer: &[u8]) -> Self {
        Self {
            bLength: u8::from_le(buffer[0]),
            bDescriptorType: u8::from_le(buffer[1]),
            bmAttributes: u8::from_le(buffer[2]),
            wDetachTimeOut: u16::from_le_bytes([buffer[3], buffer[4]]),
            wTransferSize: u16::from_le_bytes([buffer[5], buffer[6]]),
            bcdDFUVersion: u16::from_le_bytes([buffer[7], buffer[8]]),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Response received by the DFU_GETSTATUS request
#[allow(non_snake_case)]
#[derive(Debug)]
pub struct DeviceStatusResponse {
    /// An indication of the status resulting from the execution of the
    /// most recent request.
    pub bStatus: DeviceStatusCode,

    /// Minimum time, in milliseconds, that the host should wait before sending
    /// a subsequent DFU_GETSTATUS request.
    pub bwPollTimeout: u32,

    /// An indication of the state that the device is going to enter immediately
    /// following transmission of this response.
    pub bState: DeviceStateCode,

    /// Index of status description in string table.
    pub iString: u8,
}

impl DeviceStatusResponse {
    /// Creates a new device status
    pub fn new(
        status: DeviceStatusCode,
        poll_timeout: u32,
        state: DeviceStateCode,
        string_index: u8,
    ) -> Self {
        Self {
            bStatus: status,
            bwPollTimeout: poll_timeout,
            bState: state,
            iString: string_index,
        }
    }

    /// Creates a new image element from a buffer of u8 values
    pub fn from_bytes(buffer: &[u8; 6]) -> Self {
        Self::new(
            DeviceStatusCode::from_byte(buffer[0]).unwrap_or(DeviceStatusCode::errUNKNOWN),
            u32::from_le_bytes([buffer[1], buffer[2], buffer[3], 0]),
            DeviceStateCode::from_byte(buffer[4]).unwrap_or(DeviceStateCode::dfuERROR),
            u8::from_le(buffer[5]),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
    /// No device handle available, device not opened
    NoDeviceHandle,

    /// DFU functional descriptor not found.
    NoDfuFunctionalDescriptor,

    /// Status code byte does not have a valid value
    InvalidStatusCode,

    /// State code byte does not have a valid value
    InvalidStateCode,

    /// Invalid device state
    InvalidDeviceState(states::DeviceStateCode),

    /// Polling failed after retries
    TooManyGetStatusRetries,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoDeviceHandle => "No device handle.".to_string(),
                Self::NoDfuFunctionalDescriptor =>
                    "DFU functional descriptor not found.".to_string(),
                Self::InvalidStatusCode => "Invalid status code".to_string(),
                Self::InvalidStateCode => "Invalid state code".to_string(),
                Self::InvalidDeviceState(state) => format!("Invalid device state {state:?}"),
                Self::TooManyGetStatusRetries => "Too many retries when polling status".to_string(),
            }
        )
    }
}
