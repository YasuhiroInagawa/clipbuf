//! Settings persistence and change propagation (requirements 7.3, 9.1–9.3).
//!
//! `settings.json` in the app data directory holds one `Settings` object with a schema
//! version. It never contains captured item content (10.1). The pure helpers `validate`,
//! `sanitize` and `diff` carry the logic; `SettingsStore` only adds the file and a lock.

use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::Deserialize;
use serde_json::Value;

use crate::model::{
    AppError, CAPACITY_RANGE, ErrorKind, Language, NewlineMode, POLL_INTERVAL_MS_RANGE,
    SETTINGS_VERSION, Settings, TAB_WIDTH_RANGE, TransferOptions,
};

/// Which settings changed in an `update`, so the runtime can apply only what is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SettingsDiff {
    pub capacity: bool,
    pub hotkey: bool,
    pub tab_width: bool,
    pub poll_interval: bool,
    pub autostart: bool,
    pub language: bool,
    pub transfer: bool,
}

impl SettingsDiff {
    pub fn any(&self) -> bool {
        self.capacity
            || self.hotkey
            || self.tab_width
            || self.poll_interval
            || self.autostart
            || self.language
            || self.transfer
    }
}

/// Range checks for user-supplied settings. Hotkey *syntax* is validated on registration.
pub fn validate(settings: &Settings) -> Result<(), ErrorKind> {
    let ok = CAPACITY_RANGE.contains(&settings.capacity)
        && TAB_WIDTH_RANGE.contains(&settings.tab_width)
        && POLL_INTERVAL_MS_RANGE.contains(&settings.poll_interval_ms)
        && !settings.hotkey.trim().is_empty();
    if ok {
        Ok(())
    } else {
        Err(ErrorKind::InvalidSettings)
    }
}

/// Loosely typed on-disk shape: every field optional and independently parsed, so one bad
/// value does not discard the rest of the file.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Partial {
    capacity: Value,
    hotkey: Value,
    tab_width: Value,
    poll_interval_ms: Value,
    autostart: Value,
    language: Value,
    transfer: Value,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct PartialTransfer {
    keep_style: Value,
    newline: Value,
    trim: Value,
    tabs_to_spaces: Value,
    fullwidth_to_space: Value,
}

fn pick<T: serde::de::DeserializeOwned>(value: Value, valid: impl Fn(&T) -> bool, default: T) -> T {
    match serde_json::from_value::<T>(value) {
        Ok(v) if valid(&v) => v,
        _ => default,
    }
}

/// Build `Settings` from arbitrary JSON: missing or invalid fields fall back to defaults,
/// unknown keys are ignored, and the schema version is always the current one.
pub fn sanitize(value: Value) -> Settings {
    let d = Settings::default();
    let p: Partial = serde_json::from_value(value).unwrap_or_default();
    let t: PartialTransfer = serde_json::from_value(p.transfer).unwrap_or_default();
    let dt = d.transfer;
    Settings {
        version: SETTINGS_VERSION,
        capacity: pick(p.capacity, |v| CAPACITY_RANGE.contains(v), d.capacity),
        hotkey: pick(p.hotkey, |v: &String| !v.trim().is_empty(), d.hotkey),
        tab_width: pick(p.tab_width, |v| TAB_WIDTH_RANGE.contains(v), d.tab_width),
        poll_interval_ms: pick(
            p.poll_interval_ms,
            |v| POLL_INTERVAL_MS_RANGE.contains(v),
            d.poll_interval_ms,
        ),
        autostart: pick(p.autostart, |_| true, d.autostart),
        language: pick::<Option<Language>>(p.language, |_| true, d.language),
        transfer: TransferOptions {
            keep_style: pick(t.keep_style, |_| true, dt.keep_style),
            newline: pick::<NewlineMode>(t.newline, |_| true, dt.newline),
            trim: pick(t.trim, |_| true, dt.trim),
            tabs_to_spaces: pick(t.tabs_to_spaces, |_| true, dt.tabs_to_spaces),
            fullwidth_to_space: pick(t.fullwidth_to_space, |_| true, dt.fullwidth_to_space),
        },
    }
}

pub fn diff(old: &Settings, new: &Settings) -> SettingsDiff {
    SettingsDiff {
        capacity: old.capacity != new.capacity,
        hotkey: old.hotkey != new.hotkey,
        tab_width: old.tab_width != new.tab_width,
        poll_interval: old.poll_interval_ms != new.poll_interval_ms,
        autostart: old.autostart != new.autostart,
        language: old.language != new.language,
        transfer: old.transfer != new.transfer,
    }
}

#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    current: RwLock<Settings>,
}

impl SettingsStore {
    /// Open (or create with defaults) the settings file at `path`.
    /// Unreadable or corrupt content is replaced by defaults rather than failing (9.2).
    pub fn open(path: PathBuf) -> Result<Self, AppError> {
        let settings = match std::fs::read_to_string(&path) {
            Ok(text) => sanitize(serde_json::from_str(&text).unwrap_or(Value::Null)),
            Err(_) => Settings::default(),
        };
        write_atomically(&path, &settings)?;
        Ok(Self {
            path,
            current: RwLock::new(settings),
        })
    }

    /// Open the store at `<app data dir>/settings.json`.
    pub fn load(app: &tauri::AppHandle) -> Result<Self, AppError> {
        use tauri::Manager;
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|_| AppError::from(ErrorKind::SettingsIo))?;
        Self::open(dir.join("settings.json"))
    }

    pub fn get(&self) -> Settings {
        self.current.read().expect("settings lock").clone()
    }

    /// Validate, persist, then swap in `next`. On validation or I/O failure nothing changes.
    pub fn update(&self, next: Settings) -> Result<SettingsDiff, AppError> {
        validate(&next)?;
        let next = Settings {
            version: SETTINGS_VERSION,
            ..next
        };
        let mut current = self.current.write().expect("settings lock");
        write_atomically(&self.path, &next)?;
        let changed = diff(&current, &next);
        *current = next;
        Ok(changed)
    }
}

/// Write via a sibling temp file and rename so a crash never leaves a truncated file.
fn write_atomically(path: &Path, settings: &Settings) -> Result<(), AppError> {
    fn io<E>(_: E) -> AppError {
        AppError::from(ErrorKind::SettingsIo)
    }
    let dir = path.parent().ok_or_else(|| io(()))?;
    std::fs::create_dir_all(dir).map_err(io)?;
    let json = serde_json::to_string_pretty(settings).map_err(io)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(io)?;
    std::fs::rename(&tmp, path).map_err(io)
}

#[cfg(test)]
mod tests;
