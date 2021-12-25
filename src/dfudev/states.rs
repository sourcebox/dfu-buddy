//! Definitions and conversions for device status and device state codes
//!
//! Reference: [DFU 1.1 Specification](https://www.usb.org/sites/default/files/DFU_1.1.pdf)

#![allow(dead_code)]

use super::{Error, Result};

////////////////////////////////////////////////////////////////////////////////

/// Device status codes, see DFU 1.1 specification section 6.1.2
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceStatusCode {
    /// No error condition is present.
    OK = 0x00,

    /// File is not targeted for use by this device.
    errTARGET = 0x01,

    /// File is for this device but fails some vendor-specific verification test.
    errFILE = 0x02,

    /// Device is unable to write memory.
    errWRITE = 0x03,

    /// Memory erase function failed.
    errERASE = 0x04,

    /// Memory erase check failed.
    errCHECK_ERASED = 0x05,

    /// Program memory function failed.
    errPROG = 0x06,

    /// Programmed memory failed verification.
    errVERIFY = 0x07,

    /// Cannot program memory due to received address that is out of range.
    errADDRESS = 0x08,

    /// Received DFU_DNLOAD with wLength = 0, but device does
    /// not think it has all of the data yet.
    errNOTDONE = 0x09,

    /// Device’s firmware is corrupt. It cannot return to run-time
    /// (non-DFU) operations.
    errFIRMWARE = 0x0A,

    /// iString indicates a vendor-specific error.
    errVENDOR = 0x0B,

    /// Device detected unexpected USB reset signaling.
    errUSBR = 0x0C,

    /// Device detected unexpected power on reset.
    errPOR = 0x0D,

    /// Something went wrong, but the device does not know what it was.
    errUNKNOWN = 0x0E,

    /// Device stalled an unexpected request.
    errSTALLEDPKT = 0x0F,
}

impl DeviceStatusCode {
    /// Returns a status code from a byte value
    pub fn from_byte(value: u8) -> Result<Self> {
        match value {
            0x00 => Ok(Self::OK),
            0x01 => Ok(Self::errTARGET),
            0x02 => Ok(Self::errFILE),
            0x03 => Ok(Self::errWRITE),
            0x04 => Ok(Self::errERASE),
            0x05 => Ok(Self::errCHECK_ERASED),
            0x06 => Ok(Self::errPROG),
            0x07 => Ok(Self::errVERIFY),
            0x08 => Ok(Self::errADDRESS),
            0x09 => Ok(Self::errNOTDONE),
            0x0A => Ok(Self::errFIRMWARE),
            0x0B => Ok(Self::errVENDOR),
            0x0C => Ok(Self::errUSBR),
            0x0D => Ok(Self::errPOR),
            0x0E => Ok(Self::errUNKNOWN),
            0x0F => Ok(Self::errSTALLEDPKT),
            _ => Err(Box::new(Error::InvalidStatusCode)),
        }
    }

    /// Converts a status code to a byte value
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Device state codes, see DFU 1.1 specification section 6.1.2
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceStateCode {
    /// Device is running its normal application.
    appIDLE = 0,

    /// Device is running its normal application, has received the
    /// DFU_DETACH request, and is waiting for a USB reset.
    appDETACH = 1,

    /// Device is operating in the DFU mode and is waiting for requests.
    dfuIDLE = 2,

    /// Device has received a block and is waiting for the host to
    /// solicit the status via DFU_GETSTATUS.
    dfuDNLOAD_SYNC = 3,

    /// Device is programming a control-write block into its nonvolatile memories.
    dfuDNBUSY = 4,

    /// Device is processing a download operation. Expecting DFU_DNLOAD requests.
    dfuDNLOAD_IDLE = 5,

    /// Device has received the final block of firmware from the host and is
    /// waiting for receipt of DFU_GETSTATUS to begin the Manifestation phase
    dfuMANIFEST_SYNC = 6,

    /// Device is in the Manifestation phase.
    dfuMANIFEST = 7,

    /// Device has programmed its memories and is waiting for a
    /// USB reset or a power on reset.
    dfuMANIFEST_WAIT_RESET = 8,

    /// The device is processing an upload operation. Expecting
    /// DFU_UPLOAD requests.
    dfuUPLOAD_IDLE = 9,

    /// An error has occurred. Awaiting the DFU_CLRSTATUS request.
    dfuERROR = 10,
}

impl DeviceStateCode {
    /// Returns a state code from a byte value
    pub fn from_byte(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::appIDLE),
            1 => Ok(Self::appDETACH),
            2 => Ok(Self::dfuIDLE),
            3 => Ok(Self::dfuDNLOAD_SYNC),
            4 => Ok(Self::dfuDNBUSY),
            5 => Ok(Self::dfuDNLOAD_IDLE),
            6 => Ok(Self::dfuMANIFEST_SYNC),
            7 => Ok(Self::dfuMANIFEST),
            8 => Ok(Self::dfuMANIFEST_WAIT_RESET),
            9 => Ok(Self::dfuUPLOAD_IDLE),
            10 => Ok(Self::dfuERROR),
            _ => Err(Box::new(Error::InvalidStateCode)),
        }
    }

    /// Converts a state code to a byte value
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}
