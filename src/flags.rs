//! flags.rs — Fast Flags I/O
//!
//! Responsibilities:
//! - Detect the latest Roblox version folder under `%LocalAppData%\Roblox\Versions\`
//! - Read/write `ClientSettings\ClientAppSettings.json`
//! - Expose an in-memory `FlagStore` (sorted `Vec` of `Flag`)

use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};

// ─── Data Model ──────────────────────────────────────────────────────────────

/// A single Fast Flag entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flag {
    pub key: String,
    pub value: String,
    /// Tracks whether the user has edited this entry in the current session.
    pub dirty: bool,
}

// ─── FlagStore ───────────────────────────────────────────────────────────────

/// Holds all flags in memory and knows how to persist them.
pub struct FlagStore {
    /// Sorted list of flags (sorted by key ascending).
    pub flags: Vec<Flag>,
    /// The resolved path of `ClientAppSettings.json` (may not exist yet).
    pub target_path: Option<PathBuf>,
}

impl FlagStore {
    pub fn new() -> Self {
        Self {
            flags: Vec::new(),
            target_path: None,
        }
    }

    // ── Path Detection ────────────────────────────────────────────────────────

    /// Auto-detect `ClientAppSettings.json` for the active Roblox Player install.
    ///
    /// Detection strategy (in priority order):
    ///
    /// 1. **Registry** — read `HKCU\Software\Roblox\RobloxStudio` or the
    ///    RobloxPlayer uninstall key to get the exact install path.
    /// 2. **Exe scan** — walk `%LocalAppData%\Roblox\Versions\version-*\` and
    ///    find the folder that actually contains `RobloxPlayerBeta.exe`.
    ///    This is more reliable than mtime because Studio updates also touch
    ///    version folders, which would make mtime pick the wrong one.
    /// 3. **mtime fallback** — if no exe is found (e.g. portable install),
    ///    fall back to the most recently modified `version-*` folder.
    ///
    /// The `ClientSettings/` directory and `ClientAppSettings.json` file do
    /// not need to exist — they will be created by `save()`.
    pub fn detect_path() -> Result<PathBuf, String> {
        // ── 1. Registry ───────────────────────────────────────────────────────
        if let Some(path) = Self::detect_via_registry() {
            return Ok(path);
        }

        // ── 2 & 3. Filesystem scan ────────────────────────────────────────────
        let local_appdata = env::var("LOCALAPPDATA")
            .map_err(|_| "LOCALAPPDATA environment variable not found".to_string())?;

        let versions_dir = PathBuf::from(&local_appdata)
            .join("Roblox")
            .join("Versions");

        if !versions_dir.exists() {
            return Err(format!(
                "Roblox not installed (directory not found: {})",
                versions_dir.display()
            ));
        }

        // Collect all version-* subdirs with their metadata.
        let mut candidates: Vec<(PathBuf, std::time::SystemTime)> =
            fs::read_dir(&versions_dir)
                .map_err(|e| format!("Cannot read Versions dir: {e}"))?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with("version-") {
                        return None;
                    }
                    let meta = entry.metadata().ok()?;
                    if !meta.is_dir() {
                        return None;
                    }
                    Some((entry.path(), meta.modified().unwrap_or(std::time::UNIX_EPOCH)))
                })
                .collect();

        if candidates.is_empty() {
            return Err("No Roblox version folders found".to_string());
        }

        // Strategy 2: prefer the folder that has RobloxPlayerBeta.exe
        // (ignores Studio-only version folders).
        if let Some((player_dir, _)) = candidates
            .iter()
            .find(|(dir, _)| dir.join("RobloxPlayerBeta.exe").exists())
        {
            return Ok(player_dir
                .join("ClientSettings")
                .join("ClientAppSettings.json"));
        }

        // Strategy 3: mtime fallback — newest folder wins.
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(candidates[0]
            .0
            .join("ClientSettings")
            .join("ClientAppSettings.json"))
    }

    /// Try reading the Roblox Player install path from the Windows registry.
    ///
    /// Checks (in order):
    ///   HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\roblox-player  → InstallLocation
    ///   HKCU\Software\Roblox\RobloxStudio                                        → (skip — Studio)
    ///
    /// Returns `None` if the registry key doesn't exist or we're not on Windows.
    fn detect_via_registry() -> Option<PathBuf> {
        // Only compiled on Windows targets.
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;

            // Query the RobloxPlayer uninstall entry for its InstallLocation.
            // We use `reg query` (always available on Windows) to avoid adding
            // a `winreg` crate dependency.
            let output = Command::new("reg")
                .args([
                    "query",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\roblox-player",
                    "/v",
                    "InstallLocation",
                ])
                .output()
                .ok()?;

            if !output.status.success() {
                return None;
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            // reg query output format:
            //   InstallLocation    REG_SZ    C:\Users\...\Roblox\Versions\version-abc123
            for line in stdout.lines() {
                let line = line.trim();
                if line.contains("InstallLocation") {
                    // Split on "REG_SZ" and take whatever follows.
                    if let Some(pos) = line.find("REG_SZ") {
                        let location = line[pos + "REG_SZ".len()..].trim();
                        if !location.is_empty() {
                            let path = PathBuf::from(location)
                                .join("ClientSettings")
                                .join("ClientAppSettings.json");
                            return Some(path);
                        }
                    }
                }
            }
            None
        }

        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    }

    // ── Load ──────────────────────────────────────────────────────────────────

    /// Load flags from disk. Returns the path string on success.
    ///
    /// If `ClientAppSettings.json` doesn't exist, starts with empty flags
    /// (so the user can create it fresh).
    pub fn load(&mut self) -> Result<String, String> {
        let path = Self::detect_path()?;
        self.target_path = Some(path.clone());

        if !path.exists() {
            // File not found is fine — user will create flags from scratch.
            self.flags.clear();
            return Ok(format!(
                "{} (new file — will be created on Apply)",
                path.display()
            ));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;

        self.parse_json(&content)?;

        Ok(path.display().to_string())
    }

    /// Load from a user-selected file path (import preset).
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        self.parse_json(&content)
    }

    /// Parse a JSON string into `self.flags`. Replaces current flags.
    fn parse_json(&mut self, content: &str) -> Result<(), String> {
        let value: Value = serde_json::from_str(content)
            .map_err(|e| format!("Invalid JSON: {e}"))?;

        let obj = value
            .as_object()
            .ok_or_else(|| "JSON root must be an object".to_string())?;

        let mut flags: Vec<Flag> = obj
            .iter()
            .map(|(k, v)| Flag {
                key: k.clone(),
                value: value_to_string(v),
                dirty: false,
            })
            .collect();

        flags.sort_by(|a, b| a.key.cmp(&b.key));
        self.flags = flags;
        Ok(())
    }

    // ── Save ──────────────────────────────────────────────────────────────────

    /// Write `self.flags` to `ClientAppSettings.json`. Returns path on success.
    pub fn save(&mut self) -> Result<String, String> {
        let path = self
            .target_path
            .clone()
            .ok_or_else(|| "No target path set. Load or detect a Roblox version first.".to_string())?;

        // Ensure `ClientSettings/` directory exists.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory {}: {e}", parent.display()))?;
        }

        let json = self.to_json_string()?;
        fs::write(&path, json)
            .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;

        // Clear dirty flags.
        for flag in &mut self.flags {
            flag.dirty = false;
        }

        Ok(path.display().to_string())
    }

    /// Serialize flags to a pretty-printed JSON string.
    pub fn to_json_string(&self) -> Result<String, String> {
        let mut map = Map::new();
        for flag in &self.flags {
            let value = parse_value(&flag.value);
            map.insert(flag.key.clone(), value);
        }
        serde_json::to_string_pretty(&Value::Object(map))
            .map_err(|e| format!("Serialisation error: {e}"))
    }

    // ── Export preset ─────────────────────────────────────────────────────────

    /// Export current flags to a user-chosen file path.
    pub fn export_to_file(&self, path: &Path) -> Result<(), String> {
        let json = self.to_json_string()?;
        fs::write(path, json)
            .map_err(|e| format!("Cannot write preset: {e}"))
    }

    // ── Reset ─────────────────────────────────────────────────────────────────

    /// Delete `ClientAppSettings.json` (removes all flags, restores Roblox defaults).
    pub fn reset(&mut self) -> Result<(), String> {
        if let Some(path) = &self.target_path {
            if path.exists() {
                fs::remove_file(path)
                    .map_err(|e| format!("Cannot delete {}: {e}", path.display()))?;
            }
        }
        self.flags.clear();
        Ok(())
    }

    // ── Flag Mutations ────────────────────────────────────────────────────────

    /// Add or update a flag. Keeps list sorted.
    pub fn set_flag(&mut self, key: String, value: String) {
        if let Some(flag) = self.flags.iter_mut().find(|f| f.key == key) {
            if flag.value != value {
                flag.value = value;
                flag.dirty = true;
            }
        } else {
            self.flags.push(Flag { key, value, dirty: true });
            self.flags.sort_by(|a, b| a.key.cmp(&b.key));
        }
    }

    /// Remove a flag by key.
    pub fn remove_flag(&mut self, key: &str) {
        self.flags.retain(|f| f.key != key);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Convert a `serde_json::Value` to its display string (unquoted for strings).
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Best-effort parsing: try bool → number → fall back to string.
///
/// This preserves the correct JSON types when writing back, e.g.
/// `"True"` → `true` (boolean), `"42"` → `42` (number).
pub fn parse_value(s: &str) -> Value {
    // Boolean
    match s.to_lowercase().as_str() {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    // Integer
    if let Ok(n) = s.parse::<i64>() {
        return Value::Number(n.into());
    }
    // Float
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Value::Number(n);
        }
    }
    // String fallback
    Value::String(s.to_string())
}