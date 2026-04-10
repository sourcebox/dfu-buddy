//! Modal dialogs.

use eframe::egui;
use egui::{Context, Id, Modal};

use crate::AppEvent;

/// Show function to be implemented by dialogs.
pub trait ShowDialog {
    /// Shows the dialog.
    ///
    /// Returns if it should be closed and an optional event.
    fn show(&mut self, ctx: &Context) -> (bool, Option<AppEvent>);
}

/// Dialog to show an error.
pub struct ErrorDialog {
    /// Dialog title.
    title: String,

    /// Dialog message.
    message: String,

    /// Event to be returned when OK is pressed.
    ok_event: Option<AppEvent>,
}

impl ErrorDialog {
    /// Returns a new dialog.
    pub fn new(title: String, message: String, ok_event: Option<AppEvent>) -> Self {
        Self {
            title,
            message,
            ok_event,
        }
    }
}

impl ShowDialog for ErrorDialog {
    fn show(&mut self, ctx: &Context) -> (bool, Option<AppEvent>) {
        let mut event = None;

        let modal = Modal::new(Id::new("Error Dialog")).show(ctx, |ui| {
            ui.set_width(400.0);

            ui.heading(&self.title);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(&self.message);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    if ui.button("Ok").clicked() {
                        event = self.ok_event.clone();
                        ui.close();
                    }
                },
            );
        });

        (modal.should_close(), event)
    }
}
