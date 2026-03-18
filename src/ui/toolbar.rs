//! ui/toolbar.rs — Top action bar
//!
//! Contains: search box, Add button, Apply button, Reset button,
//! Import preset button, Export preset button, flag count label.

use eframe::egui::{self, Color32, RichText};
use rfd::FileDialog;

use crate::App;

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal_wrapped(|ui| {
        // ── Search ────────────────────────────────────────────────────────
        ui.label("🔍");
        ui.add(
            egui::TextEdit::singleline(&mut app.search)
                .hint_text("Search flags…")
                .desired_width(200.0),
        );

        if !app.search.is_empty() {
            if ui.small_button("✖").clicked() {
                app.search.clear();
            }
        }

        ui.separator();

        // ── Add flag ──────────────────────────────────────────────────────
        if ui
            .button(RichText::new("➕  Add Flag").color(Color32::from_rgb(100, 200, 100)))
            .clicked()
        {
            app.add_modal.open = true;
            app.add_modal.just_opened = true;
            app.add_modal.key.clear();
            app.add_modal.value.clear();
            app.add_modal.error.clear();
        }

        ui.separator();

        // ── Apply ─────────────────────────────────────────────────────────
        if ui
            .button(RichText::new("✔  Apply to Roblox").color(Color32::from_rgb(80, 160, 240)))
            .on_hover_text("Write flags to ClientAppSettings.json")
            .clicked()
        {
            app.apply();
        }

        // ── Reset ─────────────────────────────────────────────────────────
        if ui
            .button(RichText::new("🗑  Reset All").color(Color32::from_rgb(220, 80, 80)))
            .on_hover_text("Delete ClientAppSettings.json (restores Roblox defaults)")
            .clicked()
        {
            app.reset();
        }

        ui.separator();

        // ── Import preset ─────────────────────────────────────────────────
        if ui
            .button("📂  Import Preset")
            .on_hover_text("Load flags from a JSON file")
            .clicked()
        {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_title("Import Fast Flags Preset")
                .pick_file()
            {
                match app.store.load_from_file(&path) {
                    Ok(()) => {
                        app.status = crate::StatusMsg::ok(format!(
                            "✔ Imported {} flags from {}",
                            app.store.flags.len(),
                            path.display()
                        ));
                    }
                    Err(e) => {
                        app.status = crate::StatusMsg::err(format!("✘ Import failed: {e}"));
                    }
                }
            }
        }

        // ── Export preset ─────────────────────────────────────────────────
        if ui
            .button("💾  Export Preset")
            .on_hover_text("Save current flags to a JSON file")
            .clicked()
        {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("my_flags.json")
                .set_title("Export Fast Flags Preset")
                .save_file()
            {
                match app.store.export_to_file(&path) {
                    Ok(()) => {
                        app.status = crate::StatusMsg::ok(format!(
                            "✔ Exported to {}",
                            path.display()
                        ));
                    }
                    Err(e) => {
                        app.status =
                            crate::StatusMsg::err(format!("✘ Export failed: {e}"));
                    }
                }
            }
        }

        ui.separator();

        // ── Flag count ────────────────────────────────────────────────────
        let total = app.store.flags.len();
        let filtered = app
            .store
            .flags
            .iter()
            .filter(|f| flag_matches(f, &app.search))
            .count();

        if app.search.is_empty() {
            ui.label(format!("{total} flag(s)"));
        } else {
            ui.label(format!("{filtered} / {total} flag(s)"));
        }
    });
}

/// Whether a flag's key or value contains the search query (case-insensitive).
pub fn flag_matches(flag: &crate::flags::Flag, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    flag.key.to_lowercase().contains(&q) || flag.value.to_lowercase().contains(&q)
}