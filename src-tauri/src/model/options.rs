use serde::{Deserialize, Serialize};

/// How newlines are handled at transfer time (7.7, 7.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NewlineMode {
    #[default]
    Keep,
    Remove,
    Space,
}

/// Transfer toggles shown on the main window. App-wide, persisted with settings (7.1–7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOptions {
    pub keep_style: bool,
    pub newline: NewlineMode,
    pub trim: bool,
    pub tabs_to_spaces: bool,
    pub fullwidth_to_space: bool,
}

impl TransferOptions {
    /// True when at least one text transformation (anything except `keep_style`) is enabled.
    pub fn has_text_transform(&self) -> bool {
        self.newline != NewlineMode::Keep
            || self.trim
            || self.tabs_to_spaces
            || self.fullwidth_to_space
    }
}

/// Which transfer action the user chose (6.1, 6.5, 6.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransferMode {
    /// Apply the current `TransferOptions`.
    Options,
    /// Plain text only, ignoring options.
    Plain,
    /// Text plus any style data exactly as captured, ignoring options.
    Raw,
}
