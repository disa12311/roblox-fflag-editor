//! ui/table.rs — Scrollable flags table
//!
//! Renders all flags (filtered by search query) as a three-column grid:
//!   Col 0: Flag key  (read-only monospace label)
//!   Col 1: Flag value (inline TextEdit with stable Id — prevents input jitter)
//!   Col 2: Delete button

use eframe::egui::{self, Color32, Id, RichText, ScrollArea};

use crate::{ui::toolbar::flag_matches, App};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    let avail   = ui.available_width();
    let col_val = avail * 0.45;
    let col_del = avail * 0.08;

    // ── Column headers ────────────────────────────────────────────────────────
    egui::Grid::new("header_grid")
        .num_columns(3)
        .min_col_width(col_del)
        .show(ui, |ui| {
            ui.label(RichText::new("Flag Name").strong());
            ui.label(RichText::new("Value").strong());
            ui.label(""); // delete col header (empty)
            ui.end_row();
        });

    ui.separator();

    // ── Scrollable body ───────────────────────────────────────────────────────
    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Collect indices of visible flags (respects search filter).
            let indices: Vec<usize> = app
                .store
                .flags
                .iter()
                .enumerate()
                .filter(|(_, f)| flag_matches(f, &app.search))
                .map(|(i, _)| i)
                .collect();

            if indices.is_empty() {
                ui.centered_and_justified(|ui| {
                    let msg = if app.search.is_empty() {
                        "No flags yet. Click ➕ Add Flag to get started.".to_string()
                    } else {
                        format!("No flags match \"{}\"", app.search)
                    };
                    ui.label(RichText::new(msg).color(Color32::GRAY));
                });
                return;
            }

            // Track which flag to delete after the loop ends
            // (cannot mutate `app.store.flags` while iterating it).
            let mut to_delete: Option<String> = None;

            egui::Grid::new("flags_grid")
                .num_columns(3)
                .min_col_width(col_del)
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    for idx in indices {
                        let flag = &mut app.store.flags[idx];

                        // ── Key column ────────────────────────────────────────
                        let key_color = if flag.dirty {
                            Color32::from_rgb(255, 210, 80)  // yellow = unsaved edit
                        } else {
                            Color32::from_rgb(180, 220, 255) // blue   = clean
                        };
                        ui.add(
                            egui::Label::new(
                                RichText::new(&flag.key)
                                    .monospace()
                                    .color(key_color)
                                    .size(12.5),
                            )
                            .wrap_mode(egui::TextWrapMode::Extend),
                        );

                        // ── Value column ──────────────────────────────────────
                        // Use a stable Id derived from the flag key so egui can
                        // preserve focus across redraws — this prevents the
                        // "input loses focus every keystroke" jitter bug.
                        let edit_id = Id::new("flag_value").with(&flag.key);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut flag.value)
                                .id(edit_id)
                                .desired_width(col_val)
                                .font(egui::TextStyle::Monospace),
                        );
                        if resp.changed() {
                            flag.dirty = true;
                        }

                        // ── Delete column ─────────────────────────────────────
                        if ui
                            .small_button(
                                RichText::new("🗑").color(Color32::from_rgb(220, 80, 80)),
                            )
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