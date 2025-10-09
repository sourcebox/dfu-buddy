#![doc = include_str!("../README.md")]
#![windows_subsystem = "windows"]
#![warn(missing_docs)]

mod dfudev;
mod ui;
mod update;

use std::time::Duration;

use eframe::egui::{
    self, FontFamily, FontId, Margin,
    style::{Spacing, TextStyle},
    vec2,
};
use simple_logger::SimpleLogger;
use ui::modal::Modal;

use ui::{device, file};

/// Size of the native application window
const WINDOW_SIZE: egui::Vec2 = egui::vec2(850.0, 605.0);

/// Max number of frames per second
const FPS_LIMIT: u32 = 25;

////////////////////////////////////////////////////////////////////////////////

/// Starts the application
fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_resizable(false)
            .with_drag_and_drop(true),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "DFU Buddy",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.all_styles_mut(|style| {
                style.text_styles = [
                    (
                        TextStyle::Small,
                        FontId::new(11.0, FontFamily::Proportional),
                    ),
                    (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
                    (
                        TextStyle::Button,
                        FontId::new(14.0, FontFamily::Proportional),
                    ),
                    (
                        TextStyle::Heading,
                        FontId::new(18.0, FontFamily::Proportional),
                    ),
                    (
                        TextStyle::Monospace,
                        FontId::new(14.0, FontFamily::Monospace),
                    ),
                ]
                .into();
                style.spacing = Spacing {
                    item_spacing: vec2(6.0, 6.0),
                    window_margin: Margin::same(8),
                    button_padding: vec2(16.0, 5.0),
                    icon_width: 16.0,
                    ..Default::default()
                };
            });
            Ok(Box::new(App::new(cc)))
        }),
    )
    .ok();
}

////////////////////////////////////////////////////////////////////////////////

/// Main application struct with states.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    /// Vector of all availables DFU devices
    #[serde(skip)]
    devices: Option<Vec<dfudev::DfuDevice>>,

    /// Id of currently selected DFU device
    #[serde(skip)]
    device_id: Option<u64>,

    /// Instance of currently opened DFU file
    #[serde(skip)]
    dfu_file: Option<dfufile::DfuFile>,

    /// DFU files checks
    #[serde(skip)]
    dfu_file_checks: DfuFileChecks,

    /// Last path shown in the open file dialog
    file_dialog_path: Option<std::path::PathBuf>,

    /// Message channel
    #[serde(skip)]
    message_channel: (
        std::sync::mpsc::Sender<Message>,
        std::sync::mpsc::Receiver<Message>,
    ),

    /// Device update state
    #[serde(skip)]
    device_update_state: DeviceUpdateState,

    /// Zoom factor.
    zoom_factor: f32,
}

////////////////////////////////////////////////////////////////////////////////

/// Messages for application actions
#[derive(Debug, Clone)]
pub enum Message {
    /// Initialization on startup
    Init,

    /// Force rescanning of devices
    RescanDevices,

    /// Select a device with a specific id
    DeviceSelected(u64),

    /// Open the file dialog
    OpenFileDialog,

    /// Clear the selected file
    ClearFile,

    /// Open a file
    OpenFile(std::path::PathBuf),

    /// Open a message dialog.
    OpenMessageDialog {
        /// Title.
        title: String,
        /// Body content.
        body: String,
    },

    /// Start the update process in a separate thread
    StartUpdate,

    /// Send from update task when operation starts
    DeviceUpdateStarted,

    /// Send from update task when everything is finished
    DeviceUpdateFinished,

    /// Send from update task when an error has occurred
    DeviceUpdateError(String),

    /// Set a new update step
    DeviceUpdateStep(DeviceUpdateStep),

    /// Set progress for device erase operation
    DeviceEraseProgress(f32),

    /// Set progress for device program operation
    DeviceProgramProgress(f32),

    /// Set progress for device verify operation
    DeviceVerifyProgress(f32),
}

////////////////////////////////////////////////////////////////////////////////

/// Contains flags for performed checks on the selected DFU file
#[derive(Default)]
pub struct DfuFileChecks {
    /// Flag if a CRC check has been performed
    crc_checked: bool,

    /// Flag if CRC is valid
    crc_valid: bool,

    /// Flag if DFU version is accepted for the selected device
    dfu_version_valid: bool,

    /// Flag if vendor id is accepted for the selected device
    vendor_id_accepted: bool,

    /// Flag if product id is accepted for the selected device
    product_id_accepted: bool,

    /// Flag if all targets are valid
    targets_valid: bool,
}

////////////////////////////////////////////////////////////////////////////////

/// State of the device update operations
#[derive(Default)]
pub struct DeviceUpdateState {
    /// Device ready flag
    device_ready: bool,

    /// File ready flag
    file_ready: bool,

    /// Flag if everything is ready to start
    preflight_checks_passed: bool,

    /// Confirmation flag set by user checkbox
    confirmed: bool,

    /// Update in progress flag
    running: bool,

    /// Flag set after finishing without errors
    finished: bool,

    /// Current step
    step: Option<DeviceUpdateStep>,

    /// Last error
    error: Option<String>,

    /// Erase operation progress 0..1 for 0..100%
    erase_progress: f32,

    /// Program operation progress 0..1 for 0..100%
    program_progress: f32,

    /// Verify operation progress 0..1 for 0..100%
    verify_progress: f32,
}

/// Current step of update procedure
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DeviceUpdateStep {
    /// Erase operation in progress
    Erase,

    /// Program operation in progress
    Program,

    /// Verify operation in progress
    Verify,
}

////////////////////////////////////////////////////////////////////////////////

impl Default for App {
    fn default() -> Self {
        Self {
            devices: None,
            device_id: None,
            dfu_file: None,
            file_dialog_path: None,
            dfu_file_checks: DfuFileChecks::default(),
            message_channel: std::sync::mpsc::channel(),
            device_update_state: DeviceUpdateState::default(),
            zoom_factor: 1.0,
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut message_dialog = Modal::new(ctx, "message_dialog");
        message_dialog.show_dialog();

        let zoom_factor = ctx.zoom_factor();
        if self.zoom_factor != zoom_factor {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(WINDOW_SIZE * zoom_factor));
            self.zoom_factor = zoom_factor;
        }

        // Continuous updates are required for message processing, but keep frame rate limited.
        ctx.request_repaint_after(Duration::from_millis(1000 / FPS_LIMIT as u64));

        while let Ok(message) = self.message_channel.1.try_recv() {
            self.process_message(&message, ctx, &mut message_dialog);
        }

        self.device_update_state.device_ready = self.device_id.is_some();
        self.device_update_state.file_ready = self.dfu_file.is_some();
        self.device_update_state.preflight_checks_passed = self.preflight_checks();

        // Top panel with menu
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                egui::containers::menu::MenuButton::new("File").ui(ui, |ui| {
                    if ui.button("Open...").clicked() {
                        self.message_channel.0.send(Message::OpenFileDialog).ok();
                        ui.close();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
            ui.add_space(0.1);
        });

        // Bottom panel with app version
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(format!("v{}", &env!("CARGO_PKG_VERSION")));
                egui::warn_if_debug_build(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Project homepage", env!("CARGO_PKG_HOMEPAGE"));
                });
            });
            ui.add_space(0.5);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.scope(|ui| {
                if self.device_update_state.running {
                    ui.disable();
                }

                ui.add_space(5.0);

                ui::device::selection(
                    ui,
                    &self.devices,
                    &self.get_selected_device(),
                    &self.message_channel.0,
                );

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.set_height(160.0);

                    let device_info = self.get_selected_device().map(|device| &device.info);

                    device::common_info(ui, device_info);
                    device::memory_info(ui, device_info);
                });

                ui.add_space(5.0);

                ui::file::selection(ui, &self.dfu_file, &self.message_channel.0);

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.set_height(160.0);

                    file::common_info(
                        ui,
                        &self.dfu_file,
                        &mut self.dfu_file_checks,
                        self.device_id.is_some(),
                    );

                    let device_info = self.get_selected_device().map(|device| &device.info);

                    file::content_info(ui, &self.dfu_file, device_info);
                });
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.set_height(100.0);
                device::update_controls(ui, &mut self.device_update_state, &self.message_channel.0);
                ui.add_space(10.0);
                device::update_progress(ui, &self.device_update_state);
            });
        });

        // File drag-and-drop
        if !self.device_update_state.running {
            if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("file_drop_target"),
                ));

                let content_rect = ctx.input(|i| i.content_rect());
                painter.rect_filled(content_rect, 0.0, egui::Color32::from_black_alpha(192));
                painter.text(
                    content_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop DFU file top open.",
                    egui::FontId::new(16.0, egui::FontFamily::Proportional),
                    ctx.style().visuals.warn_fg_color,
                );
            }

            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                for file in &dropped_files {
                    if let Some(path) = &file.path {
                        self.message_channel
                            .0
                            .send(Message::OpenFile(path.clone()))
                            .ok();
                        break;
                    }
                }
            }
        }
    }
}

impl App {
    /// Create the application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        log::info!("USB hotplug: {}", dfudev::has_hotplug());

        app.message_channel.0.send(Message::Init).ok();

        let mut args = std::env::args();

        if args.len() > 1 {
            // First CLI argument is used as file path
            let file_path = std::path::PathBuf::from(args.nth(1).unwrap().trim());
            if file_path.exists() && file_path.is_file() {
                app.message_channel
                    .0
                    .send(Message::OpenFile(file_path))
                    .ok();
            } else {
                log::error!("File {:?} does not exist.", file_path);
            }
        }

        app
    }

    /// Process a message
    fn process_message(
        &mut self,
        message: &Message,
        ctx: &egui::Context,
        message_dialog: &mut Modal,
    ) {
        match message {
            Message::Init => {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(WINDOW_SIZE));
                self.scan_devices();
            }
            Message::RescanDevices => {
                self.scan_devices();
            }
            Message::DeviceSelected(device_id) => {
                self.device_id = Some(*device_id);
                self.match_file_against_device();
                let device = self.get_selected_device().unwrap();
                log::debug!("Selected device {}", device.info);
                self.device_update_state = DeviceUpdateState::default();
            }
            Message::OpenFileDialog => {
                self.open_file_dialog();
            }
            Message::ClearFile => {
                self.dfu_file = None;
                self.dfu_file_checks = DfuFileChecks::default();
                self.device_update_state = DeviceUpdateState::default();
            }
            Message::OpenFile(file_path) => {
                log::debug!("Opening file {:?}", file_path);
                self.open_file(file_path);
                self.match_file_against_device();
                if let Some(parent_path) = file_path.parent() {
                    self.file_dialog_path = Some(std::path::PathBuf::from(parent_path));
                }
                self.device_update_state = DeviceUpdateState::default();
            }
            Message::OpenMessageDialog { title, body } => {
                message_dialog
                    .dialog()
                    .with_title(title)
                    .with_body(body)
                    .open();
            }
            Message::DeviceUpdateStarted => {
                log::debug!("Device update started.");
                self.device_update_state = DeviceUpdateState::default();
                self.device_update_state.running = true;
                self.device_update_state.finished = false;
            }
            Message::DeviceUpdateFinished => {
                log::debug!("Device update finished.");
                self.device_update_state.running = false;
                self.device_update_state.step = None;
                self.device_update_state.finished = true;
            }
            Message::DeviceUpdateError(error) => {
                log::error!("Device update error: {}", error);
                self.device_update_state.running = false;
                self.device_update_state.error = Some(error.to_string());
            }
            Message::DeviceUpdateStep(step) => {
                log::debug!("Device update step {:?}", step);
                self.device_update_state.step = Some(*step)
            }
            Message::DeviceEraseProgress(value) => self.device_update_state.erase_progress = *value,
            Message::DeviceProgramProgress(value) => {
                self.device_update_state.program_progress = *value
            }
            Message::DeviceVerifyProgress(value) => {
                self.device_update_state.verify_progress = *value
            }
            Message::StartUpdate => {
                if !self.device_update_state.running {
                    let device_id = self.device_id.unwrap();
                    let file_path = self.dfu_file.as_ref().unwrap().path.clone();
                    let message_sender = self.message_channel.0.clone();
                    let message_sender_result = self.message_channel.0.clone();
                    std::thread::spawn(move || {
                        let result = update::full_update(device_id, file_path, message_sender);
                        match result {
                            Ok(_) => {}
                            Err(error) => {
                                message_sender_result
                                    .send(Message::DeviceUpdateError(format!("{error}")))
                                    .ok();
                            }
                        }
                    });
                } else {
                    log::error!("Update already in progress.");
                }
            }
        }
    }

    /// Find all DFU devices
    fn scan_devices(&mut self) {
        log::debug!("Scanning USB devices...");
        let devices = dfudev::DfuDevice::find(false);

        match devices {
            Ok(devices) => {
                if let Some(devices) = &devices {
                    for device in devices {
                        log::debug!("Found DFU device {}", &device.info);
                    }
                    if self.device_id.is_none() {
                        // Select the first device found
                        self.device_id = Some(devices[0].id);
                        self.match_file_against_device();
                    }
                } else {
                    log::debug!("No DFU devices found");
                    self.device_id = None;
                }
                self.devices = devices;
            }
            Err(error) => {
                log::error!("{}", error);
                self.devices = None;
                self.device_id = None;
            }
        }
    }

    /// Return reference to device with a certain id
    fn get_device(&self, id: u64) -> Option<&dfudev::DfuDevice> {
        if self.devices.is_some() {
            self.devices.as_ref().unwrap().iter().find(|&x| x.id == id)
        } else {
            None
        }
    }

    /// Return reference to currently selected device
    fn get_selected_device(&self) -> Option<&dfudev::DfuDevice> {
        match self.device_id {
            Some(device_id) => self.get_device(device_id),
            None => None,
        }
    }

    /// Open the file dialog
    fn open_file_dialog(&mut self) {
        let mut start_dir = dirs::home_dir().unwrap_or_default();

        start_dir = self
            .file_dialog_path
            .as_ref()
            .unwrap_or(&start_dir)
            .to_path_buf();

        let result = rfd::FileDialog::new()
            .add_filter("DFU files", &["dfu"])
            .set_directory(start_dir)
            .pick_file();

        if let Some(file_path) = result {
            self.message_channel
                .0
                .send(Message::OpenFile(file_path))
                .ok();
        }
    }

    /// Open a DFU file
    fn open_file(&mut self, file_path: &std::path::Path) {
        let dfu_file = dfufile::DfuFile::open(file_path);

        match dfu_file {
            Ok(mut dfu_file) => {
                self.dfu_file_checks = DfuFileChecks::default();
                let crc = dfu_file.calc_crc();
                match crc {
                    Ok(crc) => {
                        self.dfu_file_checks.crc_checked = true;
                        self.dfu_file_checks.crc_valid = crc == dfu_file.suffix.dwCRC;
                    }
                    Err(error) => {
                        log::error!("{}", error);
                    }
                }
                self.dfu_file = Some(dfu_file);
            }
            Err(error) => {
                log::error!("{}", error);
                self.message_channel
                    .0
                    .send(Message::OpenMessageDialog {
                        title: "Error opening DFU file".into(),
                        body: format!("{error}"),
                    })
                    .ok();
                self.dfu_file = None;
            }
        }
    }

    /// Match the selected file against the current device
    /// and set the file check flags accordingly
    fn match_file_against_device(&mut self) {
        if let Some(dfu_file) = &self.dfu_file {
            if let Some(device) = self.get_selected_device() {
                let device_dfu_version = device.info.dfu_version;
                let device_vendor_id = device.info.vendor_id;
                let device_product_id = device.info.product_id;
                let device_alt_settings = device.info.alt_settings.clone();
                let file_dfu_version = dfu_file.suffix.bcdDFU;
                let file_vendor_id = dfu_file.suffix.idVendor;
                let file_product_id = dfu_file.suffix.idProduct;

                self.dfu_file_checks.dfu_version_valid = file_dfu_version == device_dfu_version;

                self.dfu_file_checks.vendor_id_accepted =
                    (file_vendor_id == 0xFFFF) || (file_vendor_id == device_vendor_id);
                self.dfu_file_checks.product_id_accepted =
                    (file_product_id == 0xFFFF) || (file_product_id == device_product_id);

                match &dfu_file.content {
                    dfufile::Content::Plain => {
                        self.dfu_file_checks.targets_valid = true;
                    }
                    dfufile::Content::DfuSe(content) => {
                        self.dfu_file_checks.targets_valid = true;
                        for image in &content.images {
                            let target = device_alt_settings
                                .iter()
                                .find(|&alt| alt.0 == image.target_prefix.bAlternateSetting);
                            if target.is_none() {
                                self.dfu_file_checks.targets_valid = false;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if everything is ready to program the device
    fn preflight_checks(&self) -> bool {
        let device = self.get_selected_device();

        let checks = &self.dfu_file_checks;

        device.is_some()
            && self.dfu_file.is_some()
            && checks.crc_valid
            && checks.dfu_version_valid
            && checks.vendor_id_accepted
            && checks.product_id_accepted
            && checks.targets_valid
    }
}
