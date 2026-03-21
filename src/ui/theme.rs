//! ui/theme.rs — Dark colour palette & egui Visuals setup
//!
//! Call `apply(ctx)` once in `App::new` to set the dark theme globally.
//! All colours are defined here as constants — change them in one place.
//!
//! egui 0.33 API notes:
//!   Rounding       → CornerRadius  (same() takes u8, not f32)
//!   window_rounding→ window_corner_radius
//!   widget.rounding→ widget.corner_radius  (field, not method)
//!   Margin::same() → takes i8
//!   Margin::symmetric() → takes i8, i8

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Shadow, Stroke, TextStyle, Visuals,
};

// ─── Palette ──────────────────────────────────────────────────────────────────

pub const BG_APP:       Color32 = Color32::from_rgb( 13,  15,  23);
pub const BG_PANEL:     Color32 = Color32::from_rgb( 20,  22,  34);
pub const BG_WIDGET:    Color32 = Color32::from_rgb( 28,  31,  46);
pub const BG_HOVERED:   Color32 = Color32::from_rgb( 38,  42,  62);
pub const BG_ACTIVE:    Color32 = Color32::from_rgb( 50,  55,  80);
pub const BORDER:       Color32 = Color32::from_rgb( 45,  49,  72);
pub const BORDER_FOCUS: Color32 = Color32::from_rgb( 77, 120, 220);

pub const TEXT:         Color32 = Color32::from_rgb(220, 224, 240);
pub const TEXT_DIM:     Color32 = Color32::from_rgb(110, 116, 150);

pub const ACCENT_BLUE:  Color32 = Color32::from_rgb( 77, 158, 255);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb( 72, 199, 142);
pub const ACCENT_RED:   Color32 = Color32::from_rgb(232,  80,  80);
pub const ACCENT_GOLD:  Color32 = Color32::from_rgb(245, 197,  61);

pub const FLAG_KEY:     Color32 = Color32::from_rgb(130, 185, 255);
pub const FLAG_DIRTY:   Color32 = ACCENT_GOLD;

// ─── Helper: CornerRadius from u8 ────────────────────────────────────────────

const fn cr(r: u8) -> CornerRadius { CornerRadius::same(r) }

// ─── Apply ────────────────────────────────────────────────────────────────────

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();

    // Window / panel
    v.window_fill           = BG_PANEL;
    v.panel_fill            = BG_PANEL;
    v.window_shadow         = Shadow::NONE;
    v.window_stroke         = Stroke::new(1.0, BORDER);
    v.window_corner_radius  = cr(6);   // was: window_rounding

    v.extreme_bg_color      = BG_APP;
    v.faint_bg_color        = Color32::from_rgb(23, 26, 40);
    v.hyperlink_color       = ACCENT_BLUE;

    // Selection highlight
    v.selection.bg_fill = Color32::from_rgba_premultiplied(77, 120, 220, 80);
    v.selection.stroke  = Stroke::new(1.0, BORDER_FOCUS);

    // ── Widget states ──
    // noninteractive
    v.widgets.noninteractive.bg_fill      = BG_WIDGET;
    v.widgets.noninteractive.bg_stroke    = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke    = Stroke::new(1.0, TEXT_DIM);
    v.widgets.noninteractive.corner_radius = cr(4);  // was: .rounding field

    // inactive
    v.widgets.inactive.bg_fill      = BG_WIDGET;
    v.widgets.inactive.bg_stroke    = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke    = Stroke::new(1.5, TEXT);
    v.widgets.inactive.corner_radius = cr(4);

    // hovered
    v.widgets.hovered.bg_fill      = BG_HOVERED;
    v.widgets.hovered.bg_stroke    = Stroke::new(1.0, BORDER_FOCUS);
    v.widgets.hovered.fg_stroke    = Stroke::new(1.5, TEXT);
    v.widgets.hovered.corner_radius = cr(4);

    // active / pressed
    v.widgets.active.bg_fill      = BG_ACTIVE;
    v.widgets.active.bg_stroke    = Stroke::new(1.5, BORDER_FOCUS);
    v.widgets.active.fg_stroke    = Stroke::new(2.0, Color32::WHITE);
    v.widgets.active.corner_radius = cr(4);

    // open (combo boxes, etc.)
    v.widgets.open.bg_fill      = BG_ACTIVE;
    v.widgets.open.bg_stroke    = Stroke::new(1.0, BORDER_FOCUS);
    v.widgets.open.fg_stroke    = Stroke::new(1.5, TEXT);
    v.widgets.open.corner_radius = cr(4);

    ctx.set_visuals(v);

    // ── Typography & spacing ──────────────────────────────────────────────────
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (TextStyle::Small,     FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Body,      FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Button,    FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Heading,   FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace)),
    ]
    .into();

    style.spacing.item_spacing   = egui::vec2(8.0, 5.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    // Margin::same() takes i8 in egui 0.33
    style.spacing.window_margin  = egui::Margin::same(12);
    style.spacing.indent         = 14.0;

    ctx.set_style(style);
}