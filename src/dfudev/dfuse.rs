//! DfuSe extensions module
//!
//! References:
//! - ST UM0290 for string descriptors memory segments coding

use super::{requests, states, DfuDevice, Error, Result, TIMEOUT};

/// Command code for "Set Address Pointer"
const CMD_SET_ADDRESS_PTR: u8 = 0x21;

/// Command code for "Erase Page"
const CMD_ERASE_PAGE: u8 = 0x41;

/// Representation of a target memory segment
#[derive(Debug)]
pub struct MemorySegment {
    /// Name of the segment
    pub name: String,

    /// Vector of regions
    pub regions: Vec<MemorySegmentRegion>,
}

/// Properties of a single memory segment region.
/// Each segment can have several regions but most usually there's only one
#[derive(Debug)]
pub struct MemorySegmentRegion {
    /// First address in this region
    pub start_address: u32,

    /// Last address in this region
    pub end_address: u32,

    /// Number of sectors in this region
    pub sector_count: u32,

    /// Size of a sector in bytes
    pub sector_size: u32,

    /// Flag to mark region as readable
    pub readable: bool,

    /// Flag to mark region as writable
    pub writable: bool,

    /// Flag to mark region as erasable
    pub erasable: bool,
}

impl MemorySegment {
    /// Creates a new segment by parsing the string descriptor
    pub fn from_string_desc<T: AsRef<str>>(string_desc: T) -> Self {
        let mut regions = Vec::new();

        let mut parts: Vec<&str> = string_desc.as_ref().split('/').collect();

        // Strip of the @ at the beginning and remove trailing spaces
        let name = String::from(parts.remove(0)).trim()[1..].to_string();

        let re = regex::Regex::new(r"(\d*)\*(\d*)(\D)(\w)").unwrap();

        while parts.len() >= 2 {
            let address_str = parts.remove(0).trim_start_matches("0x");
            let mut address = u32::from_str_radix(address_str, 16).unwrap_or_default();

            let mut sectors_str: Vec<&str> = parts.remove(0).split(',').collect();

            while !sectors_str.is_empty() {
                let sector_str = sectors_str.remove(0);
                let captures = re.captures(sector_str).unwrap();

                let sector_count = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap_or_default();

                let multiplier_str = captures.get(3).unwrap().as_str();
                let multiplier = match multiplier_str {
                    "K" => 1024,
                    "M" => 1024 * 1024,
                    _ => 1,
                };
                let sector_size = captures
                    .get(2)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap_or_default()
                    * multiplier;

                let sector_type = captures.get(4).unwrap().as_str();
                let readable = matches!(sector_type, "a" | "c" | "e" | "g");
                let writable = matches!(sector_type, "d" | "e" | "f" | "g");
                let erasable = matches!(sector_type, "b" | "c" | "f" | "g");

                let region = MemorySegmentRegion {
                    start_address: address,
                    end_address: address + sector_count * sector_size - 1,
                    sector_count,
                    sector_size,
                    readable,
                    writable,
                    erasable,
                };

                regions.push(region);

                address += sector_count * sector_size;
            }
        }

        Self { name, regions }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// High-level function to set the address for subsequent uploads or downloads
pub fn set_address(device: &DfuDevice, address: u32) -> Result<()> {
    // Device must be in idle state for this operation
    device.abort_request()?;

    // Issue the request
    set_address_request(device, address)?;

    // First status response must have state dfuDNBUSY
    let status = device.getstatus_request()?;
    if status.bState != states::DeviceStateCode::dfuDNBUSY {
        return Err(Box::new(Error::InvalidDeviceState(status.bState)));
    }

    device.wait_for_status_response(status.bwPollTimeout as u64)?;

    // Abort to return to idle state, otherwise following requests can fail
    device.abort_request()?;

    Ok(())
}

/// High-level function to erase a page
pub fn erase_page(device: &DfuDevice, address: u32) -> Result<()> {
    // Device must be in idle state for this operation
    device.abort_request()?;

    // Issue the request
    erase_page_request(device, address)?;

    // First status response must have state dfuDNBUSY
    let status = device.getstatus_request()?;
    if status.bState != states::DeviceStateCode::dfuDNBUSY {
        return Err(Box::new(Error::InvalidDeviceState(status.bState)));
    }

    device.wait_for_status_response(status.bwPollTimeout as u64)?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

/// Send a SET_ADDRESS request
pub fn set_address_request(device: &DfuDevice, address: u32) -> Result<()> {
    let addr = address.to_le_bytes();
    let data = [CMD_SET_ADDRESS_PTR, addr[0], addr[1], addr[2], addr[3]];

    device.handle()?.write_control(
        requests::DFU_DNLOAD.0,
        requests::DFU_DNLOAD.1,
        0,
        0,
        &data,
        TIMEOUT,
    )?;

    Ok(())
}

/// Send a ERASE_PAGE request
pub fn erase_page_request(device: &DfuDevice, address: u32) -> Result<()> {
    let addr = address.to_le_bytes();
    let data = [CMD_ERASE_PAGE, addr[0], addr[1], addr[2], addr[3]];

    device.handle()?.write_control(
        requests::DFU_DNLOAD.0,
        requests::DFU_DNLOAD.1,
        0,
        0,
        &data,
        TIMEOUT,
    )?;

    Ok(())
}
