//! flags.rs — Fast Flags I/O
//!
//! - Auto-detect Roblox Player install path (Registry → exe scan → mtime)
//! - Read/write `ClientSettings\ClientAppSettings.json`
//! - In-memory `FlagStore` — sorted `Vec<Flag>`

use std::{
    env,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde_json::{Map, Value};

// ─── Data Model ──────────────────────────────────────────────────────────────

/// A single Fast Flag entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flag {
    pub key:   String,
    pub value: String,
    /// True when the value has been edited but not yet saved.
    pub dirty: bool,
}

// ─── FlagStore ───────────────────────────────────────────────────────────────

/// In-memory flag store — owns the sorted flag list and the target path.
#[derive(Default)]
pub struct FlagStore {
    pub flags:       Vec<Flag>,
    pub target_path: Option<PathBuf>,
}

impl FlagStore {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Path Detection ────────────────────────────────────────────────────────

    /// Auto-detect `ClientAppSettings.json` for the active Roblox Player install.
    ///
    /// Priority order:
    /// 1. **Registry** — `HKCU\…\Uninstall\roblox-player → InstallLocation`
    /// 2. **Exe scan** — find the `version-*` folder containing `RobloxPlayerBeta.exe`
    /// 3. **mtime fallback** — newest `version-*` folder by modification time
    pub fn detect_path() -> Result<PathBuf, String> {
        if let Some(path) = Self::detect_via_registry() {
            return Ok(path);
        }

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

        // Collect all version-* subdirs.
        // Edition 2024: closures in filter_map capture their upvalues precisely;
        // no behaviour change here since we only borrow inside the closure.
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

        // Strategy 2: folder with RobloxPlayerBeta.exe (not Studio).
        if let Some((dir, _)) = candidates
            .iter()
            .find(|(dir, _)| dir.join("RobloxPlayerBeta.exe").exists())
        {
            return Ok(client_settings(dir));
        }

        // Strategy 3: mtime fallback.
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(client_settings(&candidates[0].0))
    }

    /// Read `InstallLocation` from the Roblox Player uninstall registry key.
    /// Uses `reg query` (always available on Windows) — no extra crate needed.
    /// Returns `None` on non-Windows or when the key is absent.
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
                .map(|loc| client_settings(Path::new(&loc)))
        }

        #[cfg(not(target_os = "windows"))]
        None
    }

    // ── Load ──────────────────────────────────────────────────────────────────

    /// Detect path + load flags. Returns the resolved path string on success.
    /// Missing file is fine — starts with empty flags (created on first save).
    pub fn load(&mut self) -> Result<String, String> {
        let path = Self::detect_path()?;
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
        let map: Map<String, Value> = self
            .flags
            .iter()
            .map(|f| (f.key.clone(), str_to_json(&f.value)))
            .collect();
        serde_json::to_string_pretty(&Value::Object(map))
            .map_err(|e| format!("Serialisation error: {e}"))
    }

    // ── Export / Reset ────────────────────────────────────────────────────────

    pub fn export_to_file(&self, path: &Path) -> Result<(), String> {
        fs::write(path, self.to_json_string()?)
            .map_err(|e| format!("Cannot write preset: {e}"))
    }

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

    /// Insert or update a flag; keeps the list sorted by key.
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

    pub fn remove_flag(&mut self, key: &str) {
        self.flags.retain(|f| f.key != key);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build `ClientSettings/ClientAppSettings.json` path from a version directory.
fn client_settings(dir: &Path) -> PathBuf {
    dir.join("ClientSettings").join("ClientAppSettings.json")
}

/// Deserialise a JSON value to its display string (strings unquoted).
fn json_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other            => other.to_string(),
    }
}

/// Re-serialise a string value to the best-fit JSON type.
/// `"true"` → Bool, `"42"` → Number(i64), `"3.14"` → Number(f64), else String.
pub fn str_to_json(s: &str) -> Value {
    match s.to_lowercase().as_str() {
        "true"  => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _       => {}
    }
    if let Ok(n) = s.parse::<i64>()   { return Value::Number(n.into()); }
    if let Ok(f) = s.parse::<f64>()   {
        if let Some(n) = serde_json::Number::from_f64(f) { return Value::Number(n); }
    }
    Value::String(s.to_string())
}