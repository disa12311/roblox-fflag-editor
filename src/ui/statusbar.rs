//! ui/statusbar.rs — Bottom status bar

use eframe::egui::{self, Color32, RichText};

use crate::StatusMsg;

pub fn show(ui: &mut egui::Ui, status: &StatusMsg) {
    ui.horizontal(|ui| {
        if status.text.is_empty() {
            ui.label(RichText::new("Ready").color(Color32::GRAY).small());
        } else {
            let color = if status.is_error {
                Color32::from_rgb(220, 80, 80)
            } else {
                Color32::from_rgb(100, 220, 100)
            };
            ui.label(RichText::new(&status.text).color(color).small());
        }
    });
}