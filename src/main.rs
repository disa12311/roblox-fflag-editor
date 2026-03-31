//! Roblox Fast Flag Editor v2.0.0
//!
//! GUI application to view, edit, and apply Roblox Fast Flags.
//! Target: x86_64-pc-windows-gnu   Edition: Rust 2024
//!
//! Architecture
//! ─────────────
//! main.rs          — App state, eframe render loop
//! auto_detect.rs   — Roblox install path detection (Registry / exe / mtime)
//! flags.rs         — FlagStore: JSON load, save, reset, mutations
//! ui/theme.rs      — Dark colour palette + egui Visuals
//! ui/toolbar.rs    — Search, Add, Apply, Reset, Import, Export
//! ui/table.rs      — Scrollable flags grid with inline editing
//! ui/modal.rs      — "Add New Flag" dialog
//! ui/statusbar.rs  — Bottom status bar

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auto_detect;
mod flags;
mod ui;

use eframe::{NativeOptions, egui};
use egui::ViewportBuilder;
use flags::FlagStore;

/// ICO file embedded at compile time — decoded to RGBA at startup.
/// This guarantees the window icon works regardless of the working directory
/// and regardless of whether `winres` succeeded during the build.
const APP_ICON: &[u8] = include_bytes!("../assets/icon.ico");

/// Decode the embedded ICO to egui `IconData` (RGBA + dimensions).
fn load_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(APP_ICON).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width: w, height: h })
}

fn main() -> eframe::Result<()> {
    let mut viewport = ViewportBuilder::default()
        .with_title("Roblox Fast Flag Editor")
        .with_inner_size([920.0, 640.0])
        .with_min_inner_size([600.0, 400.0]);

    // Set window icon — title bar, taskbar, Alt+Tab.
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    eframe::run_native(
        "Roblox Fast Flag Editor",
        NativeOptions { viewport, ..Default::default() },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// ─── Status message ───────────────────────────────────────────────────────────

pub struct StatusMsg {
    pub text:     String,
    pub ttl:      u32,      // frames remaining; 0 = persist until overwritten
    pub is_error: bool,
}

impl Default for StatusMsg {
    fn default() -> Self {
        Self { text: String::new(), ttl: 0, is_error: false }
    }
}

impl StatusMsg {
    pub fn ok(text: impl Into<String>) -> Self {
        Self { text: text.into(), ttl: 180, is_error: false }
    }
    pub fn err(text: impl Into<String>) -> Self {
        Self { text: text.into(), ttl: 300, is_error: true }
    }
    pub fn tick(&mut self) {
        if self.ttl > 0 {
            self.ttl -= 1;
            if self.ttl == 0 { self.text.clear(); }
        }
    }
}

// ─── App state ────────────────────────────────────────────────────────────────

pub struct App {
    pub store:     FlagStore,
    pub search:    String,
    pub add_modal: ui::modal::AddModal,
    pub status:    StatusMsg,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::apply(&cc.egui_ctx);

        let mut store = FlagStore::new();
        let status = match store.load() {
            Ok(path) => StatusMsg::ok(format!("Loaded: {path}")),
            Err(e)   => StatusMsg::err(format!("Could not load flags: {e}")),
        };
        Self {
            store,
            search: String::new(),
            add_modal: ui::modal::AddModal::default(),
            status,
        }
    }

    pub fn apply(&mut self) {
        self.status = match self.store.save() {
            Ok(path) => StatusMsg::ok(format!("✔ Applied to: {path}")),
            Err(e)   => StatusMsg::err(format!("✘ Save failed: {e}")),
        };
    }

    pub fn reset(&mut self) {
        self.status = match self.store.reset() {
            Ok(())  => StatusMsg::ok("✔ Reset: ClientAppSettings.json deleted"),
            Err(e)  => StatusMsg::err(format!("✘ Reset failed: {e}")),
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.status.tick();

        let panel_frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(ui::theme::BG_PANEL)
            .inner_margin(egui::Margin::symmetric(10, 6));

        egui::TopBottomPanel::top("toolbar")
            .frame(panel_frame)
            .show(ctx, |ui| ui::toolbar::show(ui, self));

        egui::TopBottomPanel::bottom("statusbar")
            .frame(panel_frame)
            .show(ctx, |ui| ui::statusbar::show(ui, &self.status));

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).fill(ui::theme::BG_APP))
            .show(ctx, |ui| ui::table::show(ui, self));

        if self.add_modal.open {
            ui::modal::show(ctx, self);
        }
    }
}