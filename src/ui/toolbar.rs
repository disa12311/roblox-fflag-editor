//! ui/toolbar.rs — Top action bar

use eframe::egui::{self, RichText, Stroke};
use rfd::FileDialog;

use crate::{ui::theme, App};

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        // ── Search ────────────────────────────────────────────────────────
        ui.label(RichText::new("🔍").color(theme::TEXT_DIM));
        ui.add(
            egui::TextEdit::singleline(&mut app.search)
                .hint_text("Search flags…")
                .desired_width(200.0)
                .text_color(theme::TEXT),
        );
        if !app.search.is_empty() {
            if styled_button(ui, "✖", theme::TEXT_DIM).clicked() {
                app.search.clear();
            }
        }

        ui.add(egui::Separator::default().spacing(10.0));

        // ── Add flag ──────────────────────────────────────────────────────
        if styled_button(ui, "➕  Add Flag", theme::ACCENT_GREEN).clicked() {
            app.add_modal.open        = true;
            app.add_modal.just_opened = true;
            app.add_modal.key.clear();
            app.add_modal.value.clear();
            app.add_modal.error.clear();
        }

        ui.add(egui::Separator::default().spacing(10.0));

        // ── Apply ─────────────────────────────────────────────────────────
        if styled_button(ui, "✔  Apply to Roblox", theme::ACCENT_BLUE)
            .on_hover_text("Write flags to ClientAppSettings.json")
            .clicked()
        {
            app.apply();
        }

        // ── Reset ─────────────────────────────────────────────────────────
        if styled_button(ui, "🗑  Reset All", theme::ACCENT_RED)
            .on_hover_text("Delete ClientAppSettings.json (restores Roblox defaults)")
            .clicked()
        {
            app.reset();
        }

        ui.add(egui::Separator::default().spacing(10.0));

        // ── Import preset ─────────────────────────────────────────────────
        if styled_button(ui, "📂  Import", theme::TEXT)
            .on_hover_text("Load flags from a JSON file")
            .clicked()
        {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_title("Import Fast Flags Preset")
                .pick_file()
            {
                match app.store.load_from_file(&path) {
                    Ok(()) => app.status = crate::StatusMsg::ok(format!(
                        "✔ Imported {} flags from {}",
                        app.store.flags.len(),
                        path.display()
                    )),
                    Err(e) => app.status = crate::StatusMsg::err(format!("✘ Import failed: {e}")),
                }
            }
        }

        // ── Export preset ─────────────────────────────────────────────────
        if styled_button(ui, "💾  Export", theme::TEXT)
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
                    Ok(()) => app.status = crate::StatusMsg::ok(format!(
                        "✔ Exported to {}", path.display()
                    )),
                    Err(e) => app.status = crate::StatusMsg::err(format!("✘ Export failed: {e}")),
                }
            }
        }

        ui.add(egui::Separator::default().spacing(10.0));

        // ── Flag count ────────────────────────────────────────────────────
        let total    = app.store.flags.len();
        let filtered = app.store.flags.iter().filter(|f| flag_matches(f, &app.search)).count();
        let count_text = if app.search.is_empty() {
            format!("{total} flag(s)")
        } else {
            format!("{filtered} / {total}")
        };
        ui.label(RichText::new(count_text).color(theme::TEXT_DIM).small());
    });
}

/// A toolbar button with custom text colour and a subtle dark background.
fn styled_button(ui: &mut egui::Ui, label: &str, color: egui::Color32) -> egui::Response {
    let btn = egui::Button::new(RichText::new(label).color(color))
        .fill(theme::BG_WIDGET)
        .stroke(Stroke::new(1.0, theme::BORDER));
    ui.add(btn)
}

/// Whether a flag's key or value contains the search query (case-insensitive).
pub fn flag_matches(flag: &crate::flags::Flag, query: &str) -> bool {
    if query.is_empty() { return true; }
    let q = query.to_lowercase();
    flag.key.to_lowercase().contains(&q) || flag.value.to_lowercase().contains(&q)
}