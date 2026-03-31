//! flags.rs — In-memory Fast Flag store with JSON persistence
//!
//! Owns the sorted `Vec<Flag>` and the target `ClientAppSettings.json` path.
//! Path detection is delegated to `auto_detect::detect_path()`.

use std::{fs, path::{Path, PathBuf}};

use serde_json::{Map, Value};

use crate::auto_detect;

// ─── Data model ───────────────────────────────────────────────────────────────

/// A single Fast Flag entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flag {
    pub key:   String,
    pub value: String,
    /// True when the value has been edited but not yet written to disk.
    pub dirty: bool,
}

// ─── FlagStore ────────────────────────────────────────────────────────────────

/// In-memory flag store — sorted by key, backed by `ClientAppSettings.json`.
#[derive(Default)]
pub struct FlagStore {
    pub flags:       Vec<Flag>,
    pub target_path: Option<PathBuf>,
}

impl FlagStore {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Load ──────────────────────────────────────────────────────────────────

    /// Auto-detect the Roblox path, then load flags from disk.
    /// Returns the resolved path string. A missing file is fine —
    /// starts empty and will be created on the first `save()`.
    pub fn load(&mut self) -> Result<String, String> {
        let path = auto_detect::detect_path()?;
        self.target_path = Some(path.clone());

        if !path.exists() {
            self.flags.clear();
            return Ok(format!("{} (new — created on Apply)", path.display()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        self.parse_json(&content)?;
        Ok(path.display().to_string())
    }

    /// Load flags from an arbitrary file (import preset).
    /// Does not change `target_path`.
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        self.parse_json(&content)
    }

    fn parse_json(&mut self, content: &str) -> Result<(), String> {
        let root: Value = serde_json::from_str(content)
            .map_err(|e| format!("Invalid JSON: {e}"))?;
        let obj = root
            .as_object()
            .ok_or("JSON root must be an object")?;

        let mut flags: Vec<Flag> = obj
            .iter()
            .map(|(k, v)| Flag { key: k.clone(), value: json_to_str(v), dirty: false })
            .collect();
        flags.sort_by(|a, b| a.key.cmp(&b.key));
        self.flags = flags;
        Ok(())
    }

    // ── Save ──────────────────────────────────────────────────────────────────

    /// Write flags to `ClientAppSettings.json`. Creates the directory if needed.
    /// Clears all dirty markers on success.
    pub fn save(&mut self) -> Result<String, String> {
        let path = self.target_path.clone()
            .ok_or("No target path — load or detect a Roblox version first.")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create {}: {e}", parent.display()))?;
        }

        fs::write(&path, self.to_json_string()?)
            .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;

        self.flags.iter_mut().for_each(|f| f.dirty = false);
        Ok(path.display().to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        let map: Map<String, Value> = self.flags
            .iter()
            .map(|f| (f.key.clone(), str_to_json(&f.value)))
            .collect();
        serde_json::to_string_pretty(&Value::Object(map))
            .map_err(|e| format!("Serialisation error: {e}"))
    }

    // ── Export / Reset ────────────────────────────────────────────────────────

    /// Export current flags to a user-chosen file path (preset save).
    pub fn export_to_file(&self, path: &Path) -> Result<(), String> {
        fs::write(path, self.to_json_string()?)
            .map_err(|e| format!("Cannot write preset: {e}"))
    }

    /// Delete `ClientAppSettings.json` and clear the in-memory list.
    /// Roblox will use its built-in defaults on next launch.
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

    // ── Mutations ─────────────────────────────────────────────────────────────

    /// Insert a new flag or update an existing one. Keeps the list sorted.
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

// ─── JSON helpers ─────────────────────────────────────────────────────────────

/// Deserialise a JSON value to its display string (strings are unquoted).
fn json_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other            => other.to_string(),
    }
}

/// Re-serialise a user string to the best-fit JSON type:
/// `"true"/"false"` → Bool, integer string → Number(i64),
/// float string → Number(f64), anything else → String.
pub fn str_to_json(s: &str) -> Value {
    match s.to_lowercase().as_str() {
        "true"  => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _       => {}
    }
    if let Ok(n) = s.parse::<i64>() { return Value::Number(n.into()); }
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) { return Value::Number(n); }
    }
    Value::String(s.to_string())
}