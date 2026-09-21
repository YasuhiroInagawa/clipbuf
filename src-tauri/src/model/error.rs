use serde::{Deserialize, Serialize};

/// Error categories returned to the frontend. Deliberately carries no detail string so
/// that captured text or OS error messages never reach logs or the UI (10.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorKind {
    ItemNotFound,
    WriteFailed,
    ReadFailed,
    HotkeyUnavailable,
    InvalidSettings,
    CaptureUnavailable,
    SettingsIo,
}

/// Error envelope for every Tauri command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub kind: ErrorKind,
}

impl From<ErrorKind> for AppError {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl std::error::Error for AppError {}
