use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::Warning;

/// Monotonic item identifier, never reused within a process.
pub type ItemId = u64;

/// What an adapter read from the OS clipboard, before any filtering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClipboardSnapshot {
    pub text: Option<String>,
    pub html: Option<String>,
    pub rtf: Option<String>,
    /// A password-manager "do not record" marker format is present (1.5).
    pub concealed: bool,
    /// clipbuf's own marker format is present, i.e. we wrote this ourselves (1.4).
    pub own_marker: bool,
}

/// A captured item held in the in-memory buffer. Immutable after capture (6.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipItem {
    pub id: ItemId,
    pub captured_at: SystemTime,
    pub text: String,
    pub html: Option<String>,
    pub rtf: Option<String>,
    pub warnings: Vec<Warning>,
}

impl ClipItem {
    /// True when HTML or RTF data was captured alongside the text (5.1).
    pub fn has_style(&self) -> bool {
        self.html.is_some() || self.rtf.is_some()
    }

    /// Frontend representation. Style data itself is never sent; only its presence.
    pub fn to_dto(&self) -> ItemDto {
        ItemDto {
            id: self.id,
            captured_at_ms: self
                .captured_at
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            text: self.text.clone(),
            has_style: self.has_style(),
            warnings: self.warnings.clone(),
        }
    }
}

/// Item as sent to the frontend. `text` is the full text (needed for horizontal scrolling).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    pub id: ItemId,
    pub captured_at_ms: u64,
    pub text: String,
    pub has_style: bool,
    pub warnings: Vec<Warning>,
}

/// Result of `transfer_item` (6.7, 7.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOutcome {
    /// Style was kept, so the enabled text transformations were not applied.
    pub skipped_transforms: bool,
}
