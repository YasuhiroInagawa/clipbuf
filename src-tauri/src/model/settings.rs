use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::TransferOptions;

/// Schema version written to `settings.json` for future migrations.
pub const SETTINGS_VERSION: u32 = 1;
pub const CAPACITY_RANGE: RangeInclusive<usize> = 1..=200;
pub const TAB_WIDTH_RANGE: RangeInclusive<u8> = 1..=16;
pub const POLL_INTERVAL_MS_RANGE: RangeInclusive<u32> = 50..=2000;

/// UI language. `None` in `Settings::language` means "follow the OS" (12.2, 12.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    Ja,
    En,
}

/// Persisted user settings (9.1, 9.2). Never contains captured item content (10.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    /// Number of items kept (2.1, 2.6). Range: `CAPACITY_RANGE`.
    pub capacity: usize,
    /// Global shortcut in Tauri's accelerator syntax (8.2).
    pub hotkey: String,
    /// Spaces per tab for `tabs_to_spaces` (7.10). Range: `TAB_WIDTH_RANGE`.
    pub tab_width: u8,
    /// Clipboard poll interval where polling is used (macOS, Wayland). Range: `POLL_INTERVAL_MS_RANGE`.
    pub poll_interval_ms: u32,
    /// Launch at OS login (9.5).
    pub autostart: bool,
    /// UI language override; `None` follows the OS (12.4).
    pub language: Option<Language>,
    /// Wrap long lines in the full-text preview instead of scrolling sideways (4.11, 4.12).
    pub preview_wrap: bool,
    /// Transfer toggles, app-wide (7.3).
    pub transfer: TransferOptions,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            capacity: 20,
            // Physical-key spelling, the same form the settings recorder produces.
            hotkey: "Alt+Shift+KeyV".to_string(),
            tab_width: 4,
            poll_interval_ms: 200,
            autostart: false,
            language: None,
            preview_wrap: true,
            transfer: TransferOptions::default(),
        }
    }
}
