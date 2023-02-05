//! Device update operations

use anyhow::{anyhow, Result};

use crate::{dfudev, DeviceUpdateStep, Message};

/// Perform a full update on the device (erase, program, verify).
///
/// This function is executed in a separate thread and communicates with
/// the main thread via messages
pub fn full_update(
    device_id: u64,
    file_path: std::path::PathBuf,
    message_sender: std::sync::mpsc::Sender<Message>,
) -> Result<()> {
    message_sender.send(Message::DeviceUpdateStarted)?;
    erase_device(device_id, &file_path, &message_sender)?;
    program_device(device_id, &file_path, &message_sender)?;
    verify_device(device_id, &file_path, &message_sender)?;
    message_sender.send(Message::DeviceUpdateFinished)?;

    Ok(())
}

/// Erase the data in the device.
fn erase_device(
    device_id: u64,
    file_path: &std::path::Path,
    message_sender: &std::sync::mpsc::Sender<Message>,
) -> Result<()> {
    // Set the step so UI knows it
    message_sender
        .send(Message::DeviceUpdateStep(DeviceUpdateStep::Erase))
        .ok();

    // Find the device by its id and open it
    let mut device = dfudev::DfuDevice::find_by_id(device_id)?.unwrap();
    device.open()?;

    // Make sure device is in idle state before operations start
    device.abort_request()?;

    // Make sure status is OK
    while let Ok(status) = device.getstatus_request() {
        if status.bStatus == dfudev::DeviceStatusCode::OK {
            break;
        } else {
            device.clrstatus_request()?;
        }
    }

    let file = dfufile::DfuFile::open(file_path)?;

    match &file.content {
        dfufile::Content::Plain => {
            log::warn!("Plain DFU does not support separate erase. Skipped.");
        }
        dfufile::Content::DfuSe(content) => {
            let num_images = content.images.len();

            for (image_no, image) in content.images.iter().enumerate() {
                let alt_setting = image.target_prefix.bAlternateSetting;
                let target = device
                    .info
                    .alt_settings
                    .iter()
                    .find(|&alt| alt.0 == alt_setting);

                if let Some(target) = target {
                    let memory_segment = dfudev::dfuse::MemorySegment::from_string_desc(&target.1);
                    log::debug!(
                        "Found target \"{}\" for alt setting {}",
                        memory_segment.name,
                        target.0,
                    );

                    let num_elements = image.image_elements.len();

                    for (element_no, element) in image.image_elements.iter().enumerate() {
                        log::debug!(
                            "Reading element at address 0x{:08X}, size {}",
                            element.dwElementAddress,
                            element.dwElementSize
                        );
                        let start_address = element.dwElementAddress;
                        let end_address = start_address + element.dwElementSize;
                        let region = memory_segment.regions.iter().find(|x| {
                            x.start_address <= start_address
                                && x.end_address >= end_address
                                && x.erasable
                        });

                        if let Some(region) = region {
                            let sector_size = region.sector_size;
                            let num_sectors = (end_address - start_address) / sector_size;
                            log::debug!("Memory region found, sector size is {}", sector_size);
                            let mut erase_address = start_address / sector_size * sector_size;
                            let mut sector_no = 0;

                            while erase_address <= end_address {
                                log::debug!("Erasing sector at 0x{:08X}", erase_address);

                                dfudev::dfuse::erase_page(&device, erase_address)?;

                                let progress = (sector_no as f32) / (num_sectors as f32)
                                    * ((image_no + 1) as f32)
                                    / (num_images as f32)
                                    * ((element_no + 1) as f32)
                                    / (num_elements as f32);
                                message_sender
                                    .send(Message::DeviceEraseProgress(progress))
                                    .ok();

                                erase_address += sector_size;
                                sector_no += 1;
                            }
                        } else {
                            return Err(anyhow!(Error::MemoryRegionNotFound(
                                start_address,
                                end_address,
                            )));
                        }
                    }
                } else {
                    return Err(anyhow!(Error::TargetNotFound(alt_setting)));
                }
            }
        }
    }

    // Final cleanup
    device.abort_request()?;
    device.close();

    Ok(())
}

/// Downloads the data to the device.
fn program_device(
    device_id: u64,
    file_path: &std::path::Path,
    message_sender: &std::sync::mpsc::Sender<Message>,
) -> Result<()> {
    // Set the step so UI knows it
    message_sender
        .send(Message::DeviceUpdateStep(DeviceUpdateStep::Program))
        .ok();

    // Find the device by its id and open it
    let mut device = dfudev::DfuDevice::find_by_id(device_id)?.unwrap();
    device.open()?;

    // Make sure device is in idle state before operations start
    device.abort_request()?;

    // Make sure status is OK
    while let Ok(status) = device.getstatus_request() {
        if status.bStatus == dfudev::DeviceStatusCode::OK {
            break;
        } else {
            device.clrstatus_request()?;
        }
    }

    let mut file = dfufile::DfuFile::open(file_path)?;

    match &file.content {
        dfufile::Content::Plain => {
            return Err(anyhow!(Error::PlainDfuNotSupported));
        }
        dfufile::Content::DfuSe(content) => {
            let num_images = content.images.len();

            for (image_no, image) in content.images.iter().enumerate() {
                let alt_setting = image.target_prefix.bAlternateSetting;
                let target = device
                    .info
                    .alt_settings
                    .iter()
                    .find(|&alt| alt.0 == alt_setting);

                if let Some(target) = target {
                    let memory_segment = dfudev::dfuse::MemorySegment::from_string_desc(&target.1);
                    let transfer_size = memory_segment
                        .regions
                        .iter()
                        .min_by_key(|x| x.sector_size)
                        .unwrap()
                        .sector_size;
                    let transfer_size =
                        std::cmp::min(transfer_size as u16, device.info.dfu_transfer_size);
                    log::debug!(
                        "Found target \"{}\" for alt setting {}. Transfer size is {} bytes",
                        memory_segment.name,
                        target.0,
                        transfer_size
                    );

                    let num_elements = image.image_elements.len();

                    for (element_no, element) in image.image_elements.iter().enumerate() {
                        log::debug!(
                            "Reading element at address 0x{:08X}, size {}",
                            element.dwElementAddress,
                            element.dwElementSize
                        );
                        let start_address = element.dwElementAddress;
                        let end_address = start_address + element.dwElementSize;
                        let mut write_address = start_address;

                        dfudev::dfuse::set_address(&device, write_address)?;

                        let mut block_no = 0;
                        let num_blocks = (end_address - start_address) / transfer_size as u32;

                        while write_address < end_address {
                            let chunk_size =
                                std::cmp::min(transfer_size as u32, end_address - write_address);

                            let mut file_data = vec![0; chunk_size as usize];
                            element.read_at(
                                &mut file.file,
                                write_address - start_address,
                                &mut file_data,
                            )?;

                            log::debug!(
                                "Programming block {} with {} bytes at address 0x{:08X}",
                                block_no,
                                chunk_size,
                                write_address
                            );

                            device.download_request(block_no + 2, &file_data)?;

                            // First status response must have state dfuDNBUSY
                            let status = device.getstatus_request()?;
                            if status.bState != dfudev::states::DeviceStateCode::dfuDNBUSY {
                                return Err(anyhow!(dfudev::Error::InvalidDeviceState(
                                    status.bState,
                                )));
                            }

                            device.wait_for_status_response(status.bwPollTimeout as u64)?;

                            log::debug!("Block no {} written", block_no);

                            let progress = (block_no as f32) / (num_blocks as f32)
                                * ((image_no + 1) as f32)
                                / (num_images as f32)
                                * ((element_no + 1) as f32)
                                / (num_elements as f32);
                            message_sender
                                .send(Message::DeviceProgramProgress(progress))
                                .ok();

                            write_address += chunk_size;
                            block_no += 1;
                        }
                    }
                } else {
                    return Err(anyhow!(Error::TargetNotFound(alt_setting)));
                }
            }
        }
    }

    // Final cleanup
    device.abort_request()?;
    device.close();

    Ok(())
}

/// Verifys the data in the device.
fn verify_device(
    device_id: u64,
    file_path: &std::path::Path,
    message_sender: &std::sync::mpsc::Sender<Message>,
) -> Result<()> {
    // Set the step so UI knows it
    message_sender
        .send(Message::DeviceUpdateStep(DeviceUpdateStep::Verify))
        .ok();

    // Find the device by its id and open it
    let mut device = dfudev::DfuDevice::find_by_id(device_id)?.unwrap();
    device.open()?;

    // Make sure device is in idle state before operations start
    device.abort_request()?;

    // Make sure status is OK
    while let Ok(status) = device.getstatus_request() {
        if status.bStatus == dfudev::DeviceStatusCode::OK {
            break;
        } else {
            device.clrstatus_request()?;
        }
    }

    let mut file = dfufile::DfuFile::open(file_path)?;

    match &file.content {
        dfufile::Content::Plain => {
            return Err(anyhow!(Error::PlainDfuNotSupported));
        }
        dfufile::Content::DfuSe(content) => {
            let num_images = content.images.len();

            for (image_no, image) in content.images.iter().enumerate() {
                let alt_setting = image.target_prefix.bAlternateSetting;
                let target = device
                    .info
                    .alt_settings
                    .iter()
                    .find(|&alt| alt.0 == alt_setting);

                if let Some(target) = target {
                    let memory_segment = dfudev::dfuse::MemorySegment::from_string_desc(&target.1);
                    let transfer_size = memory_segment
                        .regions
                        .iter()
                        .min_by_key(|x| x.sector_size)
                        .unwrap()
                        .sector_size;
                    let transfer_size =
                        std::cmp::min(transfer_size as u16, device.info.dfu_transfer_size);
                    log::debug!(
                        "Found target \"{}\" for alt setting {}. Transfer size is {} bytes",
                        memory_segment.name,
                        target.0,
                        transfer_size
                    );

                    let num_elements = image.image_elements.len();

                    for (element_no, element) in image.image_elements.iter().enumerate() {
                        log::debug!(
                            "Reading element at address 0x{:08X}, size {}",
                            element.dwElementAddress,
                            element.dwElementSize
                        );
                        let start_address = element.dwElementAddress;
                        let end_address = start_address + element.dwElementSize;
                        let mut read_address = start_address;

                        dfudev::dfuse::set_address(&device, read_address)?;

                        let mut block_no = 0;
                        let num_blocks = (end_address - start_address) / transfer_size as u32;

                        while read_address < end_address {
                            let chunk_size =
                                std::cmp::min(transfer_size as u32, end_address - read_address);

                            let mut device_data = vec![0; chunk_size as usize];
                            device.upload_request(block_no + 2, &mut device_data)?;

                            let mut file_data = vec![0; chunk_size as usize];
                            element.read_at(
                                &mut file.file,
                                read_address - start_address,
                                &mut file_data,
                            )?;

                            if device_data != file_data {
                                return Err(anyhow!(Error::VerificationFailed(read_address)));
                            }

                            let progress = (block_no as f32) / (num_blocks as f32)
                                * ((image_no + 1) as f32)
                                / (num_images as f32)
                                * ((element_no + 1) as f32)
                                / (num_elements as f32);
                            message_sender
                                .send(Message::DeviceVerifyProgress(progress))
                                .ok();

                            read_address += chunk_size;
                            block_no += 1;
                        }
                    }
                } else {
                    return Err(anyhow!(Error::TargetNotFound(alt_setting)));
                }
            }
        }
    }

    // Final cleanup
    device.abort_request()?;
    device.close();

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Error {
    /// Target not found for an alternate setting
    TargetNotFound(u8),

    /// Memory region not found for an address range
    MemoryRegionNotFound(u32, u32),

    /// Verification error
    VerificationFailed(u32),

    /// Plain DFU is not supported yet
    PlainDfuNotSupported,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::TargetNotFound(alt_setting) =>
                    format!("No target found for alt setting {}.", alt_setting),
                Self::MemoryRegionNotFound(start_address, end_address) => format!(
                    "No memory region found with address 0x{:08X}..0x{:08X}",
                    start_address, end_address
                ),
                Self::VerificationFailed(address) =>
                    format!("Verification failed at address 0x{:08X}.", address),
                Self::PlainDfuNotSupported => "Plain DFU devices are not supported yet".to_string(),
            }
        )
    }
}
