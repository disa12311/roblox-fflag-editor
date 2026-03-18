# Roblox Fast Flags Editor

A Windows GUI application built with Rust + egui to edit Roblox's `ClientAppSettings.json` Fast Flags.

## Features

- 🔍 **Search / filter** flags by name or value
- ➕ **Add** new flags with auto-type detection (bool / int / float / string)
- ✏️ **Edit** flag values inline in the table
- 🗑 **Delete** individual flags
- ✔ **Apply to Roblox** — writes directly to the latest Roblox version's `ClientSettings\ClientAppSettings.json`
- 📂 **Import Preset** — load a JSON flag preset file
- 💾 **Export Preset** — save current flags to a JSON file
- 🗑 **Reset All** — deletes `ClientAppSettings.json` (restores Roblox defaults)

## Fast Flags File Location

The app auto-detects the **latest** Roblox version folder:

```
%LocalAppData%\Roblox\Versions\version-<hash>\ClientSettings\ClientAppSettings.json
```

If the file doesn't exist, it will be created when you click **Apply to Roblox**.

## Building

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- MinGW-w64 for GNU target: `x86_64-w64-mingw32-gcc`

### Windows (native)

```powershell
# Add the target if not already present
rustup target add x86_64-pc-windows-gnu

# Build release binary
cargo build --release --target x86_64-pc-windows-gnu
```

Output: `target\x86_64-pc-windows-gnu\release\roblox-flag-editor.exe`

### Cross-compile from Linux

```bash
# Install MinGW toolchain
sudo apt install gcc-mingw-w64-x86-64

# Add Rust target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

### Alternatively (Windows MSVC target)

If you prefer MSVC, remove `.cargo/config.toml` and run:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

## Usage

1. Run `roblox-flag-editor.exe`
2. The app loads existing flags from your Roblox installation automatically
3. Add / edit / delete flags as needed
4. Click **✔ Apply to Roblox** to save
5. Launch Roblox — the flags are now active

## Flag Value Types

Values are auto-detected when saved:

| Input string | JSON type written |
|---|---|
| `true` / `false` | boolean |
| `42` | integer |
| `3.14` | float |
| anything else | string |

## Common Fast Flags

A few examples to get started (not pre-loaded, add manually):

| Flag | Value | Effect |
|---|---|---|
| `FIntRenderLocalLightUpdatesMax` | `8` | Reduce local light updates |
| `FIntTaskSchedulerTargetFps` | `144` | Uncap FPS |
| `FLogNetwork` | `0` | Disable network logging |
| `FFlagDebugGraphicsPreferVulkan` | `true` | Use Vulkan renderer |

> ⚠️ Fast Flags are internal Roblox engine knobs. Use at your own risk — incorrect values may cause crashes or bans.
