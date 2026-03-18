//! ui/modal.rs — "Add New Flag" modal dialog
//!
//! Renders a centred overlay window with:
//!   • Key input   (must be non-empty)
//!   • Value input (any string; auto-typed to bool/number on save)
//!   • Add button  (also triggered by Enter)
//!   • Cancel button

use eframe::egui::{self, Color32, Id, Key, RichText};

use crate::App;

// ─── State ────────────────────────────────────────────────────────────────────

/// State for the Add-flag modal, owned by `App`.
#[derive(Default)]
pub struct AddModal {
    pub open: bool,
    pub key: String,
    pub value: String,
    pub error: String,
    /// True only on the very first frame after opening — used to auto-focus
    /// the Key field exactly once without stealing focus every frame.
    pub just_opened: bool,
}

// ─── Stable widget IDs ────────────────────────────────────────────────────────
// Fixed IDs prevent egui from reassigning widget identity across redraws,
// which would cause focus to be lost after every keystroke.

const ID_KEY:   &str = "add_modal_key";
const ID_VALUE: &str = "add_modal_value";

// ─── Render ───────────────────────────────────────────────────────────────────

pub fn show(ctx: &egui::Context, app: &mut App) {
    let mut open = app.add_modal.open;

    egui::Window::new("Add New Fast Flag")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([420.0, 200.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // ── Key ───────────────────────────────────────────────────
                ui.label("Flag Name (key):");
                ui.add(
                    egui::TextEdit::singleline(&mut app.add_modal.key)
                        .id(Id::new(ID_KEY))
                        .hint_text("e.g. FFlagSomething")
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(6.0);

                // ── Value ─────────────────────────────────────────────────
                ui.label("Value:");
                ui.add(
                    egui::TextEdit::singleline(&mut app.add_modal.value)
                        .id(Id::new(ID_VALUE))
                        .hint_text("true / false / 42 / some-string")
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(4.0);

                // ── Inline type hint ──────────────────────────────────────
                ui.label(
                    RichText::new(format!("Type: {}", infer_type_label(&app.add_modal.value)))
                        .small()
                        .color(Color32::GRAY),
                );

                // ── Error message ─────────────────────────────────────────
                if !app.add_modal.error.is_empty() {
                    ui.label(
                        RichText::new(&app.add_modal.error)
                            .color(Color32::from_rgb(220, 80, 80)),
                    );
                }

                ui.add_space(8.0);

                // ── Buttons ───────────────────────────────────────────────
                ui.horizontal(|ui| {
                    let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));
                    if ui
                        .button(RichText::new("➕  Add Flag").color(Color32::from_rgb(100, 200, 100)))
                        .clicked()
                        || enter_pressed
                    {
                        try_add(app);
                    }
                    if ui.button("Cancel").clicked() {
                        app.add_modal.open = false;
                    }
                });

                // Auto-focus Key field only on the first frame after opening.
                // Calling request_focus() every frame would steal focus from
                // the Value field whenever the user tries to type there.
                if app.add_modal.just_opened {
                    ctx.memory_mut(|mem| mem.request_focus(Id::new(ID_KEY)));
                    app.add_modal.just_opened = false;
                }
            });
        });

    app.add_modal.open = open;
}

// ─── Logic ────────────────────────────────────────────────────────────────────

fn try_add(app: &mut App) -> bool {
    let key   = app.add_modal.key.trim().to_string();
    let value = app.add_modal.value.clone();

    if key.is_empty() {
        app.add_modal.error = "Flag name cannot be empty.".to_string();
        return false;
    }

    let verb = if app.store.flags.iter().any(|f| f.key == key) { "Updated" } else { "Added" };
    app.store.set_flag(key.clone(), value.clone());
    app.status = crate::StatusMsg::ok(format!("{verb} flag: {key} = {value}"));
    app.add_modal.open = false;
    true
}

fn infer_type_label(s: &str) -> &'static str {
    if s.is_empty() { return "string (empty)"; }
    match s.to_lowercase().as_str() {
        "true" | "false" => return "boolean",
        _ => {}
    }
    if s.parse::<i64>().is_ok() { return "integer"; }
    if s.parse::<f64>().is_ok() { return "float"; }
    "string"
}