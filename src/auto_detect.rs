//! auto_detect.rs — Roblox Player install path detection
//!
//! Tries three strategies in order:
//!
//! 1. **Registry** — `HKCU\…\Uninstall\roblox-player → InstallLocation`
//!    Exact path from the official Roblox installer; most reliable.
//!
//! 2. **Exe scan** — walk `%LocalAppData%\Roblox\Versions\version-*\`
//!    and pick the folder that actually contains `RobloxPlayerBeta.exe`.
//!    Correctly ignores Studio-only version folders.
//!
//! 3. **mtime fallback** — pick the most recently modified `version-*`
//!    folder. Works for portable / non-standard installs.
//!
//! All strategies return the path to:
//!   `<version_dir>\ClientSettings\ClientAppSettings.json`

use std::{env, fs, path::{Path, PathBuf}, time::SystemTime};

/// Detect the `ClientAppSettings.json` path for the active Roblox Player.
/// The file does not need to exist — it will be created by `FlagStore::save()`.
pub fn detect_path() -> Result<PathBuf, String> {
    // Strategy 1 — Windows registry
    if let Some(path) = detect_via_registry() {
        return Ok(path);
    }

    // Strategies 2 & 3 — filesystem scan
    let local_appdata = env::var("LOCALAPPDATA")
        .map_err(|_| "LOCALAPPDATA environment variable not found".to_string())?;

    let versions_dir = PathBuf::from(local_appdata)
        .join("Roblox")
        .join("Versions");

    if !versions_dir.exists() {
        return Err(format!(
            "Roblox not installed (not found: {})",
            versions_dir.display()
        ));
    }

    let mut candidates: Vec<(PathBuf, SystemTime)> = fs::read_dir(&versions_dir)
        .map_err(|e| format!("Cannot read Versions dir: {e}"))?
        .filter_map(|res| {
            let entry = res.ok()?;
            let name  = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("version-") { return None; }
            let meta  = entry.metadata().ok()?;
            if !meta.is_dir() { return None; }
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            Some((entry.path(), mtime))
        })
        .collect();

    if candidates.is_empty() {
        return Err("No Roblox version folders found".to_string());
    }

    // Strategy 2 — find the folder with RobloxPlayerBeta.exe
    if let Some((dir, _)) = candidates
        .iter()
        .find(|(dir, _)| dir.join("RobloxPlayerBeta.exe").exists())
    {
        return Ok(client_settings_path(dir));
    }

    // Strategy 3 — newest folder by modification time
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(client_settings_path(&candidates[0].0))
}

/// Read `InstallLocation` from the Roblox Player uninstall entry in the
/// Windows registry. Uses the built-in `reg query` command — no extra crate.
/// Returns `None` on non-Windows targets or when the key is absent.
fn detect_via_registry() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let out = Command::new("reg")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\roblox-player",
                "/v",
                "InstallLocation",
            ])
            .output()
            .ok()?;

        if !out.status.success() { return None; }

        String::from_utf8_lossy(&out.stdout)
            .lines()
            .find(|l| l.contains("InstallLocation"))
            .and_then(|l| l.find("REG_SZ").map(|pos| l[pos + 6..].trim().to_owned()))
            .filter(|s| !s.is_empty())
            .map(|loc| client_settings_path(Path::new(&loc)))
    }

    #[cfg(not(target_os = "windows"))]
    None
}

/// Append `ClientSettings\ClientAppSettings.json` to a version directory path.
pub fn client_settings_path(version_dir: &Path) -> PathBuf {
    version_dir
        .join("ClientSettings")
        .join("ClientAppSettings.json")
}