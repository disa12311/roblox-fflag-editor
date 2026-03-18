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

    /// Detect `ClientAppSettings.json` path for the latest Roblox version.
    ///
    /// Search order:
    /// 1. `%LocalAppData%\Roblox\Versions\version-*\ClientSettings\ClientAppSettings.json`
    ///    → picks the most recently modified version folder.
    /// 2. Fallback: construct path even if it doesn't exist (so Save can create it).
    pub fn detect_path() -> Result<PathBuf, String> {
        let local_appdata = env::var("LOCALAPPDATA")
            .map_err(|_| "LOCALAPPDATA environment variable not found".to_string())?;

        let versions_dir = PathBuf::from(&local_appdata)
            .join("Roblox")
            .join("Versions");

        if !versions_dir.exists() {
            return Err(format!(
                "Roblox not found at {}",
                versions_dir.display()
            ));
        }

        // Collect all `version-*` subdirectories, pick newest by modification time.
        let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = fs::read_dir(&versions_dir)
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
                let mtime = meta.modified().ok()?;
                Some((entry.path(), mtime))
            })
            .collect();

        if candidates.is_empty() {
            return Err("No Roblox version folders found in Versions directory".to_string());
        }

        // Sort descending by modification time → newest first.
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let latest = &candidates[0].0;
        let settings_path = latest
            .join("ClientSettings")
            .join("ClientAppSettings.json");

        Ok(settings_path)
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
