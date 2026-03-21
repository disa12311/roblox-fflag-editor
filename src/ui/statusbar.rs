//! ui/statusbar.rs — Bottom status bar

use eframe::egui::{self, RichText};

use crate::{ui::theme, StatusMsg};

pub fn show(ui: &mut egui::Ui, status: &StatusMsg) {
    ui.horizontal(|ui| {
        let (icon, color) = if status.text.is_empty() {
            ("●", theme::TEXT_DIM)
        } else if status.is_error {
            ("✘", theme::ACCENT_RED)
        } else {
            ("✔", theme::ACCENT_GREEN)
        };

        ui.label(RichText::new(icon).color(color).small());

        let msg = if status.text.is_empty() { "Ready" } else { &status.text };
        ui.label(RichText::new(msg).color(color).small());
    });
}