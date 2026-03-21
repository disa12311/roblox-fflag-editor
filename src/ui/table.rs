//! ui/table.rs — Scrollable flags table

use eframe::egui::{self, Id, Margin, RichText, ScrollArea, Stroke};

use crate::{ui::{theme, toolbar::flag_matches}, App};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    let avail   = ui.available_width();
    let col_val = avail * 0.44;
    let col_del = 36.0;

    // ── Column headers ────────────────────────────────────────────────────────
    // Frame::new() replaces deprecated Frame::none() in egui 0.33
    egui::Frame::new()
        .fill(theme::BG_PANEL)
        .inner_margin(Margin::symmetric(6, 4))   // i8 args in egui 0.33
        .show(ui, |ui| {
            egui::Grid::new("header_grid")
                .num_columns(3)
                .min_col_width(col_del)
                .show(ui, |ui| {
                    ui.label(RichText::new("Flag Name").color(theme::TEXT_DIM).small().strong());
                    ui.label(RichText::new("Value").color(theme::TEXT_DIM).small().strong());
                    ui.label("");
                    ui.end_row();
                });
        });

    ui.add(egui::Separator::default().spacing(0.0));

    // ── Scrollable body ───────────────────────────────────────────────────────
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let indices: Vec<usize> = app
                .store.flags.iter().enumerate()
                .filter(|(_, f)| flag_matches(f, &app.search))
                .map(|(i, _)| i)
                .collect();

            if indices.is_empty() {
                ui.add_space(40.0);
                ui.centered_and_justified(|ui| {
                    let msg = if app.search.is_empty() {
                        "No flags yet — click ➕ Add Flag to get started."
                    } else {
                        "No flags match the search query."
                    };
                    ui.label(RichText::new(msg).color(theme::TEXT_DIM));
                });
                return;
            }

            let mut to_delete: Option<String> = None;

            egui::Grid::new("flags_grid")
                .num_columns(3)
                .min_col_width(col_del)
                .striped(true)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    for idx in indices {
                        let flag = &mut app.store.flags[idx];

                        // ── Key ───────────────────────────────────────────
                        let key_color = if flag.dirty { theme::FLAG_DIRTY } else { theme::FLAG_KEY };
                        ui.add(
                            egui::Label::new(
                                RichText::new(&flag.key)
                                    .monospace()
                                    .color(key_color)
                                    .size(12.5),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );

                        // ── Value ─────────────────────────────────────────
                        // Stable Id prevents focus loss / jitter each frame
                        let edit_id = Id::new("flag_value").with(&flag.key);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut flag.value)
                                .id(edit_id)
                                .desired_width(col_val)
                                .font(egui::TextStyle::Monospace)
                                .text_color(theme::TEXT),
                        );
                        if resp.changed() { flag.dirty = true; }

                        // ── Delete ────────────────────────────────────────
                        let del_btn = egui::Button::new(
                            RichText::new("✕").color(theme::ACCENT_RED).size(11.0),
                        )
                        .fill(theme::BG_WIDGET)
                        .stroke(Stroke::new(1.0, theme::BORDER))
                        .min_size(egui::vec2(26.0, 22.0));

                        if ui.add(del_btn)
                            .on_hover_text(format!("Remove: {}", flag.key))
                            .clicked()
                        {
                            to_delete = Some(flag.key.clone());
                        }

                        ui.end_row();
                    }
                });

            if let Some(key) = to_delete {
                app.store.remove_flag(&key);
                app.status = crate::StatusMsg::ok(format!("Removed flag: {key}"));
            }
        });
}