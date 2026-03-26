//! ui/modal.rs — "Add New Flag" modal dialog

use eframe::egui::{self, CornerRadius, Id, Key, Margin, RichText, Stroke};

use crate::{App, ui::theme};

#[derive(Default)]
pub struct AddModal {
    pub open:        bool,
    pub key:         String,
    pub value:       String,
    pub error:       String,
    pub just_opened: bool,
}

const ID_KEY:   &str = "add_modal_key";
const ID_VALUE: &str = "add_modal_value";

pub fn show(ctx: &egui::Context, app: &mut App) {
    let mut open = app.add_modal.open;

    let frame = egui::Frame::window(&ctx.style())
        .fill(theme::BG_PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(16));

    egui::Window::new(RichText::new("Add New Fast Flag").color(theme::TEXT).strong())
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([440.0, 210.0])
        .frame(frame)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            // ── Key ───────────────────────────────────────────────────────
            ui.label(RichText::new("Flag Name (key)").color(theme::TEXT_DIM).small());
            ui.add(
                egui::TextEdit::singleline(&mut app.add_modal.key)
                    .id(Id::new(ID_KEY))
                    .hint_text("e.g. FFlagSomething")
                    .font(egui::TextStyle::Monospace)
                    .text_color(theme::TEXT)
                    .desired_width(f32::INFINITY),
            );

            ui.add_space(4.0);

            // ── Value ─────────────────────────────────────────────────────
            ui.label(RichText::new("Value").color(theme::TEXT_DIM).small());
            ui.add(
                egui::TextEdit::singleline(&mut app.add_modal.value)
                    .id(Id::new(ID_VALUE))
                    .hint_text("true / false / 42 / some-string")
                    .font(egui::TextStyle::Monospace)
                    .text_color(theme::TEXT)
                    .desired_width(f32::INFINITY),
            );

            // ── Type hint ─────────────────────────────────────────────────
            ui.label(
                RichText::new(format!("→ {}", type_label(&app.add_modal.value)))
                    .small()
                    .color(theme::ACCENT_BLUE),
            );

            // ── Error ─────────────────────────────────────────────────────
            if !app.add_modal.error.is_empty() {
                ui.label(
                    RichText::new(&app.add_modal.error)
                        .color(theme::ACCENT_RED)
                        .small(),
                );
            }

            ui.add_space(8.0);

            // ── Buttons ───────────────────────────────────────────────────
            ui.horizontal(|ui| {
                let add_btn = egui::Button::new(
                    RichText::new("➕  Add Flag").color(theme::ACCENT_GREEN),
                )
                .fill(theme::BG_WIDGET)
                .stroke(Stroke::new(1.0, theme::ACCENT_GREEN));

                if ui.add(add_btn).clicked()
                    || ui.input(|i| i.key_pressed(Key::Enter))
                {
                    try_add(app);
                }

                let cancel = egui::Button::new(
                    RichText::new("Cancel").color(theme::TEXT_DIM),
                )
                .fill(theme::BG_WIDGET)
                .stroke(Stroke::new(1.0, theme::BORDER));

                if ui.add(cancel).clicked() {
                    app.add_modal.open = false;
                }
            });

            // Focus Key field once on open.
            if app.add_modal.just_opened {
                ctx.memory_mut(|m| m.request_focus(Id::new(ID_KEY)));
                app.add_modal.just_opened = false;
            }
        });

    app.add_modal.open = open;
}

fn try_add(app: &mut App) -> bool {
    let key   = app.add_modal.key.trim().to_string();
    let value = app.add_modal.value.clone();

    if key.is_empty() {
        app.add_modal.error = "Flag name cannot be empty.".into();
        return false;
    }

    let verb = if app.store.flags.iter().any(|f| f.key == key) { "Updated" } else { "Added" };
    app.store.set_flag(key.clone(), value.clone());
    app.status = crate::StatusMsg::ok(format!("{verb} flag: {key} = {value}"));
    app.add_modal.open = false;
    true
}

fn type_label(s: &str) -> &'static str {
    if s.is_empty() { return "string (empty)"; }
    match s.to_lowercase().as_str() {
        "true" | "false" => return "boolean",
        _ => {}
    }
    if s.parse::<i64>().is_ok() { return "integer"; }
    if s.parse::<f64>().is_ok() { return "float"; }
    "string"
}