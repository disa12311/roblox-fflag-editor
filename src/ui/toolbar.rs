//! ui/toolbar.rs — Top action bar

use eframe::egui::{self, RichText, Stroke};
use rfd::FileDialog;

use crate::{App, ui::theme};

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
        if !app.search.is_empty()
            && btn(ui, "✖", theme::TEXT_DIM).clicked()
        {
            app.search.clear();
        }

        sep(ui);

        // ── Add ───────────────────────────────────────────────────────────
        if btn(ui, "➕  Add Flag", theme::ACCENT_GREEN).clicked() {
            app.add_modal = crate::ui::modal::AddModal {
                open:        true,
                just_opened: true,
                ..Default::default()
            };
        }

        sep(ui);

        // ── Apply ─────────────────────────────────────────────────────────
        if btn(ui, "✔  Apply to Roblox", theme::ACCENT_BLUE)
            .on_hover_text("Write flags to ClientAppSettings.json")
            .clicked()
        {
            app.apply();
        }

        // ── Reset ─────────────────────────────────────────────────────────
        if btn(ui, "🗑  Reset All", theme::ACCENT_RED)
            .on_hover_text("Delete ClientAppSettings.json (restores Roblox defaults)")
            .clicked()
        {
            app.reset();
        }

        sep(ui);

        // ── Import ────────────────────────────────────────────────────────
        if btn(ui, "📂  Import", theme::TEXT)
            .on_hover_text("Load flags from a JSON preset file")
            .clicked()
        {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_title("Import Fast Flags Preset")
                .pick_file()
            {
                app.status = match app.store.load_from_file(&path) {
                    Ok(()) => crate::StatusMsg::ok(format!(
                        "✔ Imported {} flags from {}",
                        app.store.flags.len(),
                        path.display()
                    )),
                    Err(e) => crate::StatusMsg::err(format!("✘ Import failed: {e}")),
                };
            }
        }

        // ── Export ────────────────────────────────────────────────────────
        if btn(ui, "💾  Export", theme::TEXT)
            .on_hover_text("Save current flags to a JSON preset file")
            .clicked()
        {
            if let Some(path) = FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_file_name("my_flags.json")
                .set_title("Export Fast Flags Preset")
                .save_file()
            {
                app.status = match app.store.export_to_file(&path) {
                    Ok(())  => crate::StatusMsg::ok(format!("✔ Exported to {}", path.display())),
                    Err(e)  => crate::StatusMsg::err(format!("✘ Export failed: {e}")),
                };
            }
        }

        sep(ui);

        // ── Count ─────────────────────────────────────────────────────────
        let total    = app.store.flags.len();
        let filtered = app.store.flags.iter().filter(|f| matches_query(f, &app.search)).count();
        let label = if app.search.is_empty() {
            format!("{total} flag(s)")
        } else {
            format!("{filtered} / {total}")
        };
        ui.label(RichText::new(label).color(theme::TEXT_DIM).small());
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn btn(ui: &mut egui::Ui, label: &str, color: egui::Color32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(color))
            .fill(theme::BG_WIDGET)
            .stroke(Stroke::new(1.0, theme::BORDER)),
    )
}

fn sep(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(10.0));
}

pub fn matches_query(flag: &crate::flags::Flag, query: &str) -> bool {
    if query.is_empty() { return true; }
    let q = query.to_lowercase();
    flag.key.to_lowercase().contains(&q) || flag.value.to_lowercase().contains(&q)
}