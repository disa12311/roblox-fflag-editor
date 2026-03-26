//! ui/statusbar.rs — Bottom status bar

use eframe::egui::{self, RichText};

use crate::{StatusMsg, ui::theme};

pub fn show(ui: &mut egui::Ui, status: &StatusMsg) {
    ui.horizontal(|ui| {
        let (icon, color) = match (status.text.is_empty(), status.is_error) {
            (true,  _    ) => ("●", theme::TEXT_DIM),
            (false, true ) => ("✘", theme::ACCENT_RED),
            (false, false) => ("✔", theme::ACCENT_GREEN),
        };
        let msg = if status.text.is_empty() { "Ready" } else { status.text.as_str() };
        ui.label(RichText::new(format!("{icon}  {msg}")).color(color).small());
    });
}