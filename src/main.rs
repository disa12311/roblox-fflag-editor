//! Roblox Fast Flags Editor
//!
//! A GUI application to view, edit, and apply Roblox Fast Flags stored in
//! `%LocalAppData%\Roblox\Versions\*\ClientSettings\ClientAppSettings.json`.
//!
//! # Architecture
//! - `main.rs`         — Entry point, app state, egui render loop
//! - `flags.rs`        — Flag loading, saving, path detection
//! - `ui/mod.rs`       — UI helper widgets
//! - `ui/toolbar.rs`   — Top toolbar (search, add, reset, import/export)
//! - `ui/table.rs`     — Flags table with edit/delete
//! - `ui/modal.rs`     — Add-flag modal dialog

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console on Windows release

mod flags;
mod ui;

use eframe::{egui, NativeOptions};
use egui::ViewportBuilder;
use flags::FlagStore;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Roblox Fast Flags Editor")
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Roblox Fast Flags Editor",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// ─── App State ───────────────────────────────────────────────────────────────

/// Root application state.
pub struct App {
    /// The in-memory flag store (key → value pairs).
    store: FlagStore,

    /// Current search query for filtering flags.
    search: String,

    /// State for the "Add New Flag" modal.
    add_modal: ui::modal::AddModal,

    /// Transient status message shown at the bottom.
    status: StatusMsg,
}

/// A status message with an optional expiry (frame counter).
struct StatusMsg {
    text: String,
    /// Countdown frames until the message clears (0 = permanent until overwritten)
    ttl: u32,
    is_error: bool,
}

impl Default for StatusMsg {
    fn default() -> Self {
        Self {
            text: String::new(),
            ttl: 0,
            is_error: false,
        }
    }
}

impl StatusMsg {
    fn ok(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ttl: 180, // ~3 seconds at 60fps
            is_error: false,
        }
    }

    fn err(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ttl: 300,
            is_error: true,
        }
    }

    fn tick(&mut self) {
        if self.ttl > 0 {
            self.ttl -= 1;
            if self.ttl == 0 {
                self.text.clear();
            }
        }
    }
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut store = FlagStore::new();
        let status = match store.load() {
            Ok(path) => StatusMsg::ok(format!("Loaded from: {}", path)),
            Err(e) => StatusMsg::err(format!("Could not load flags: {e}")),
        };

        Self {
            store,
            search: String::new(),
            add_modal: ui::modal::AddModal::default(),
            status,
        }
    }

    /// Apply changes: write the JSON file to the Roblox ClientSettings path.
    fn apply(&mut self) {
        self.status = match self.store.save() {
            Ok(path) => StatusMsg::ok(format!("✔ Applied to: {}", path)),
            Err(e) => StatusMsg::err(format!("✘ Save failed: {e}")),
        };
    }

    /// Reset: delete the ClientAppSettings.json file entirely.
    fn reset(&mut self) {
        self.status = match self.store.reset() {
            Ok(_) => {
                StatusMsg::ok("✔ Reset: ClientAppSettings.json deleted (all flags cleared)")
            }
            Err(e) => StatusMsg::err(format!("✘ Reset failed: {e}")),
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.status.tick();

        // ── Top toolbar ──
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui::toolbar::show(ui, self);
        });

        // ── Status bar ──
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui::statusbar::show(ui, &self.status);
        });

        // ── Main table ──
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::table::show(ui, self);
        });

        // ── Add-flag modal (rendered on top) ──
        if self.add_modal.open {
            ui::modal::show(ctx, self);
        }
    }
}