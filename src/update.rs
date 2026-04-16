//! Device update operations

use anyhow::{Result, anyhow};

use crate::{AppEvent, DeviceUpdateStep, dfudev};

/// Perform a full update on the device (erase, program, verify).
///
/// This function is executed in a separate thread and communicates with
/// the main thread via events.
pub fn full_update(
    device_id: u64,
    file_path: std::path::PathBuf,
    event_sender: std::sync::mpsc::Sender<AppEvent>,
) -> Result<()> {
    event_sender.send(AppEvent::DeviceUpdateStarted)?;
    erase_device(device_id, &file_path, &event_sender)?;
    program_device(device_id, &file_path, &event_sender)?;
    verify_device(device_id, &file_path, &event_sender)?;
    event_sender.send(AppEvent::DeviceUpdateFinished)?;

    Ok(())
}

/// Erase the data in the device.
fn erase_device(
    device_id: u64,
    file_path: &std::path::Path,
    event_sender: &std::sync::mpsc::Sender<AppEvent>,
) -> Result<()> {
    // Set the step so UI knows it
    event_sender
        .send(AppEvent::DeviceUpdateStep(DeviceUpdateStep::Erase))
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
            event_sender.send(AppEvent::DeviceEraseProgress(1.0)).ok();
        }
        dfufile::Content::Dfuse(content) => {
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
                                event_sender
                                    .send(AppEvent::DeviceEraseProgress(progress))
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
    event_sender: &std::sync::mpsc::Sender<AppEvent>,
) -> Result<()> {
    // Set the step so UI knows it
    event_sender
        .send(AppEvent::DeviceUpdateStep(DeviceUpdateStep::Program))
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
            let content_size = file.size()? - dfufile::SUFFIX_LENGTH as u64;
            let transfer_size = (device.info.dfu_transfer_size as u64).min(content_size) as u16;
            let num_blocks = (content_size / transfer_size as u64 + 1) as u16;

            for block_no in 0..num_blocks {
                let chunk_size = transfer_size
                    .min((content_size - block_no as u64 * transfer_size as u64) as u16);
                let mut file_data = vec![0; chunk_size as usize];
                let file_pos = block_no as u64 * transfer_size as u64;
                file.read_raw_at(file_pos, &mut file_data)?;

                log::debug!("Programming block {} with {} bytes.", block_no, chunk_size);

                device.wait_for_download_idle()?;
                device.download_request(block_no, &file_data)?;

                let status = device.getstatus_request()?;
                device.wait_for_status_response(status.bwPollTimeout as u64)?;

                log::debug!("Block no {} written", block_no);

                let progress = (block_no as f32) / ((num_blocks - 1) as f32);
                event_sender
                    .send(AppEvent::DeviceProgramProgress(progress))
                    .ok();
            }

            // Send zero-length request to indicate completed transfer.
            device.wait_for_download_idle()?;
            device.download_request(num_blocks, &[])?;
            device.getstatus_request()?;
        }
        dfufile::Content::Dfuse(content) => {
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
                        std::cmp::min(transfer_size, device.info.dfu_transfer_size as u32);
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
                        let num_blocks = (end_address - start_address) / transfer_size;

                        while write_address < end_address {
                            let chunk_size =
                                std::cmp::min(transfer_size, end_address - write_address);

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

                            device.wait_for_download_idle()?;
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
                            event_sender
                                .send(AppEvent::DeviceProgramProgress(progress))
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

/// Verifies the data in the device.
fn verify_device(
    device_id: u64,
    file_path: &std::path::Path,
    event_sender: &std::sync::mpsc::Sender<AppEvent>,
) -> Result<()> {
    // Set the step so UI knows it
    event_sender
        .send(AppEvent::DeviceUpdateStep(DeviceUpdateStep::Verify))
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
            let content_size = file.size()? - dfufile::SUFFIX_LENGTH as u64;
            let transfer_size = (device.info.dfu_transfer_size as u64).min(content_size) as u16;
            let num_blocks = (content_size / transfer_size as u64 + 1) as u16;

            for block_no in 0..num_blocks {
                let chunk_size = transfer_size
                    .min((content_size - block_no as u64 * transfer_size as u64) as u16);

                device.wait_for_upload_idle()?;
                let mut device_data = vec![0; chunk_size as usize];
                device.upload_request(block_no, &mut device_data)?;

                let mut file_data = vec![0; chunk_size as usize];
                let file_pos = block_no as u64 * transfer_size as u64;
                file.read_raw_at(file_pos, &mut file_data)?;

                if device_data != file_data {
                    return Err(anyhow!(Error::VerificationFailedBlock(block_no as u32)));
                }

                let progress = (block_no as f32) / ((num_blocks - 1) as f32);
                event_sender
                    .send(AppEvent::DeviceVerifyProgress(progress))
                    .ok();
            }
        }
        dfufile::Content::Dfuse(content) => {
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
                        std::cmp::min(transfer_size, device.info.dfu_transfer_size as u32);
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
                        let num_blocks = (end_address - start_address) / transfer_size;

                        while read_address < end_address {
                            let chunk_size =
                                std::cmp::min(transfer_size, end_address - read_address);

                            device.wait_for_upload_idle()?;
                            let mut device_data = vec![0; chunk_size as usize];
                            device.upload_request(block_no + 2, &mut device_data)?;

                            let mut file_data = vec![0; chunk_size as usize];
                            element.read_at(
                                &mut file.file,
                                read_address - start_address,
                                &mut file_data,
                            )?;

                            if device_data != file_data {
                                return Err(anyhow!(Error::VerificationFailedAddress(
                                    read_address
                                )));
                            }

                            let progress = (block_no as f32) / (num_blocks as f32)
                                * ((image_no + 1) as f32)
                                / (num_images as f32)
                                * ((element_no + 1) as f32)
                                / (num_elements as f32);
                            event_sender
                                .send(AppEvent::DeviceVerifyProgress(progress))
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

    /// Verification failed at specific address.
    VerificationFailedAddress(u32),

    /// Verification failed at specific block.
    VerificationFailedBlock(u32),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::TargetNotFound(alt_setting) =>
                    format!("No target found for alt setting {alt_setting}."),
                Self::MemoryRegionNotFound(start_address, end_address) => format!(
                    "No memory region found with address 0x{start_address:08X}..0x{end_address:08X}"
                ),
                Self::VerificationFailedAddress(address) =>
                    format!("Verification failed at address 0x{address:08X}."),
                Self::VerificationFailedBlock(block) =>
                    format!("Verification failed at block {block}."),
            }
        )
    }
}
