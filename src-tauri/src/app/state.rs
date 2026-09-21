//! Process-wide runtime state shared by the capture service, commands and window wiring.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::buffer::Buffer;
use crate::clipboard::ClipboardPort;
use crate::clipboard::marker::LastWrite;
use crate::settings::SettingsStore;

pub struct AppState {
    /// Captured items; the Rust side is the source of truth, the frontend holds a copy.
    pub buffer: Mutex<Buffer>,
    pub settings: SettingsStore,
    pub clipboard: Box<dyn ClipboardPort>,
    /// Hash of the last text we wrote, for self-write exclusion (1.4).
    pub last_write: LastWrite,
    read_failures: AtomicU32,
}

impl AppState {
    pub fn new(settings: SettingsStore, clipboard: impl Into<Box<dyn ClipboardPort>>) -> Self {
        let capacity = settings.get().capacity;
        Self {
            buffer: Mutex::new(Buffer::new(capacity)),
            settings,
            clipboard: clipboard.into(),
            last_write: LastWrite::default(),
            read_failures: AtomicU32::new(0),
        }
    }

    pub fn consecutive_read_failures(&self) -> u32 {
        self.read_failures.load(Ordering::Relaxed)
    }

    /// Returns the new consecutive failure count.
    pub(crate) fn note_read_failure(&self) -> u32 {
        self.read_failures.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub(crate) fn note_read_success(&self) {
        self.read_failures.store(0, Ordering::Relaxed);
    }
}
