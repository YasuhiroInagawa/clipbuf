//! Self-write detection (requirement 1.4).
//!
//! Two independent signals are combined by the capture service:
//! 1. A private clipboard format written alongside every transfer. Its name differs per
//!    platform convention (UTI on macOS, registered format on Windows, MIME elsewhere).
//! 2. A hash of the last text we wrote, for environments where the private format is dropped.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Mutex;

/// Uniform Type Identifier style name used with NSPasteboard.
pub const MARKER_FORMAT_MACOS: &str = "org.clipbuf.marker";
/// Registered clipboard format name on Windows (`RegisterClipboardFormat`).
pub const MARKER_FORMAT_WINDOWS: &str = "clipbuf-marker";
/// MIME type / X11 target used on X11 and Wayland.
pub const MARKER_FORMAT_MIME: &str = "application/x-clipbuf-marker";

/// The marker format name for the platform this binary runs on.
pub fn marker_format() -> &'static str {
    if cfg!(target_os = "macos") {
        MARKER_FORMAT_MACOS
    } else if cfg!(target_os = "windows") {
        MARKER_FORMAT_WINDOWS
    } else {
        MARKER_FORMAT_MIME
    }
}

/// True when any platform spelling of the marker format is among `formats`.
pub fn has_marker_format<'a>(formats: impl IntoIterator<Item = &'a str>) -> bool {
    formats
        .into_iter()
        .any(|f| f == MARKER_FORMAT_MACOS || f == MARKER_FORMAT_WINDOWS || f == MARKER_FORMAT_MIME)
}

/// Hash of the text most recently written by clipbuf. Shared between the transfer command
/// (which records) and the capture service (which compares).
#[derive(Debug, Default)]
pub struct LastWrite {
    hash: Mutex<Option<u64>>,
}

impl LastWrite {
    pub fn record(&self, text: &str) {
        *self.hash.lock().expect("LastWrite lock") = Some(hash_text(text));
    }

    pub fn matches(&self, text: &str) -> bool {
        *self.hash.lock().expect("LastWrite lock") == Some(hash_text(text))
    }

    pub fn clear(&self) {
        *self.hash.lock().expect("LastWrite lock") = None;
    }
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}
