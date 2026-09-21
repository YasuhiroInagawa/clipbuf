//! Clipboard port and OS-specific adapters. The only place that touches OS clipboard APIs.
//!
//! `ClipboardPort` is the OS-independent contract used by the capture service and the
//! transfer command. Adapters live behind it (`clipboard_rs` for Windows / macOS / X11,
//! `wayland` for Linux Wayland) and a `FakeClipboard` exists for tests.

use std::sync::mpsc::Sender;

pub mod clipboard_rs;
pub mod conceal;
pub mod marker;

#[cfg(test)]
pub mod fake;

use crate::model::{CaptureCapability, ClipboardSnapshot};

/// Notification from an adapter's watcher. Content is read separately via `read`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardEvent {
    Changed,
}

/// What to place on the clipboard. The adapter adds clipbuf's own marker format (1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WritePayload<'a> {
    pub text: &'a str,
    pub html: Option<&'a str>,
    pub rtf: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipError {
    /// No usable clipboard in this environment (e.g. no display server).
    Unavailable,
    Read,
    Write,
    WatchAlreadyStarted,
}

impl std::fmt::Display for ClipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ClipError {}

/// OS-independent clipboard access.
///
/// Implementations must be cheap to share across threads: watcher callbacks arrive on an
/// OS thread and only send `ClipboardEvent::Changed`; the capture service reads on its own.
pub trait ClipboardPort: Send + Sync {
    /// Start observing the clipboard, sending `Changed` on every change. A second call
    /// returns `ClipError::WatchAlreadyStarted`.
    fn start_watch(&self, sender: Sender<ClipboardEvent>) -> Result<(), ClipError>;

    /// Read the current contents. `text` is `None` when no text format is present.
    fn read(&self) -> Result<ClipboardSnapshot, ClipError>;

    /// Write text (plus html / rtf when given) together with clipbuf's marker format.
    fn write(&self, payload: WritePayload<'_>) -> Result<(), ClipError>;

    /// What this adapter can observe in the current environment (11.3–11.6).
    fn capability(&self) -> CaptureCapability;
}

#[cfg(test)]
mod tests;
