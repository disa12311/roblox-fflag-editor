# Roblox Fast Flag Editor

A Windows desktop app built with **Rust 2024 + egui 0.33** to edit Roblox's
`ClientAppSettings.json` Fast Flags — with a fully dark-themed GUI, smart
auto-detection of the active Roblox installation, and zero runtime dependencies.

## Features

| | |
|---|---|
| 🔍 | **Search / filter** flags by name or value in real time |
| ➕ | **Add** new flags — auto-detects JSON type (bool / int / float / string) |
| ✏️ | **Edit** flag values inline — no focus-loss jitter |
| 🗑 | **Delete** individual flags |
| ✔ | **Apply to Roblox** — writes to `ClientAppSettings.json` instantly |
| 📂 | **Import Preset** — load a saved JSON flag file |
| 💾 | **Export Preset** — save current flags to a JSON file |
| ↺ | **Reset All** — deletes `ClientAppSettings.json` (restores Roblox defaults) |
| 🎨 | **Dark theme** — custom palette matching egui 0.33 |
| 🖼 | **App icon** — embedded in `.exe` and set at runtime |

## Auto-Detection

The app finds the active Roblox Player installation automatically — no
configuration needed. Three strategies are tried in order:

1. **Windows Registry** — reads `InstallLocation` from the Roblox uninstall key
   (`HKCU\...\Uninstall\roblox-player`). Most accurate.
2. **Exe scan** — walks `%LocalAppData%\Roblox\Versions\version-*\` and picks
   the folder containing `RobloxPlayerBeta.exe`. Correctly skips Studio folders.
3. **mtime fallback** — picks the most recently modified `version-*` folder.
   Works for non-standard / portable installs.

`ClientAppSettings.json` is created automatically on first **Apply** if it
doesn't already exist.

## Project Structure

```
roblox-fast-flag-editor/
├── Cargo.toml
├── build.rs                  ← embeds icon into .exe via winres
├── .cargo/config.toml        ← default target: x86_64-pc-windows-gnu
├── .gitignore
├── assets/
│   └── icon.ico              ← multi-size icon (16–256 px)
└── src/
    ├── main.rs               ← App state, eframe loop, runtime icon
    ├── auto_detect.rs        ← Roblox path detection (Registry / exe / mtime)
    ├── flags.rs              ← FlagStore: load, save, reset, JSON helpers
    └── ui/
        ├── mod.rs
        ├── theme.rs          ← dark colour palette + egui Visuals
        ├── toolbar.rs        ← Search, Add, Apply, Reset, Import, Export
        ├── table.rs          ← scrollable flags grid, inline editing
        ├── modal.rs          ← Add New Flag dialog
        └── statusbar.rs      ← bottom status bar (auto-clears after ~3 s)
```

## Flag Value Types

Values are auto-typed when written to JSON:

| You type | Written as |
|---|---|
| `true` / `false` | `bool` |
| `42` | `integer` |
| `3.14` | `float` |
| anything else | `string` |

## Building

### Prerequisites

- [Rust toolchain](https://rustup.rs/) — stable channel
- MinGW-w64: `x86_64-w64-mingw32-gcc` and `x86_64-w64-mingw32-windres`

### Windows (native)

```powershell
rustup target add x86_64-pc-windows-gnu
cargo build --release
# → target\x86_64-pc-windows-gnu\release\roblox-fast-flag-editor.exe
```

### Cross-compile from Linux

```bash
sudo apt install gcc-mingw-w64-x86-64
rustup target add x86_64-pc-windows-gnu
cargo build --release
# → target/x86_64-pc-windows-gnu/release/roblox-fast-flag-editor.exe
```

> The `.cargo/config.toml` sets the default target, so `--target` is optional.

### MSVC fallback

Delete `.cargo/config.toml` then:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Note: `build.rs` will skip `winres` icon embedding on MSVC without additional
setup. The runtime icon (set via `viewport.with_icon()`) always works regardless.

## Usage

1. Run `roblox-fast-flag-editor.exe`
2. Flags load automatically from your Roblox installation
3. Add / edit / delete flags as needed
4. Click **✔ Apply to Roblox** to write the file
5. Launch Roblox — flags are active immediately

## Common Fast Flags

A few examples to get started (add manually via ➕):

| Flag | Value | Effect |
|---|---|---|
| `FIntTaskSchedulerTargetFps` | `144` | Uncap framerate |
| `FFlagDebugGraphicsPreferVulkan` | `true` | Use Vulkan renderer |
| `FIntRenderLocalLightUpdatesMax` | `8` | Reduce local light update load |
| `FLogNetwork` | `0` | Disable network logging |
| `FFlagDisableNewIGMinDUA` | `true` | Disable in-game overlay |

> ⚠️ Fast Flags are internal Roblox engine knobs. Use at your own risk —
> incorrect values may cause crashes. Roblox may patch or remove flags at any time.