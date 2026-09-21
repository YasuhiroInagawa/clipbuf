//! `ClipboardPort` adapter over the `clipboard-rs` crate (Windows, macOS, Linux/X11).
//!
//! Change detection: Windows uses a clipboard listener, macOS polls `changeCount` every
//! `poll_interval`, X11 uses XFixes — all inside `clipboard-rs`'s watcher, which runs on
//! its own thread and only forwards `ClipboardEvent::Changed`. Content is read on demand.

use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::time::Duration;

use clipboard_rs::{
    Clipboard, ClipboardContent, ClipboardContext, ClipboardHandler, ClipboardWatcher,
    ClipboardWatcherContext, WatcherShutdown,
};

use super::conceal::has_concealed_format;
use super::marker::{has_marker_format, marker_format};
use super::{ClipError, ClipboardEvent, ClipboardPort, WritePayload};
use crate::model::{CaptureCapability, ClipboardSnapshot};

pub struct ClipboardRsAdapter {
    /// `ClipboardContext` is `Send` on every platform but only `Sync` on some; the mutex
    /// makes the adapter uniformly shareable.
    ctx: Mutex<ClipboardContext>,
    poll_interval: Duration,
    watcher: Mutex<Option<WatcherShutdown>>,
    capability: CaptureCapability,
}

impl ClipboardRsAdapter {
    /// `poll_interval` is used where the platform has no change notification (macOS).
    pub fn new(poll_interval: Duration) -> Result<Self, ClipError> {
        let ctx = ClipboardContext::new().map_err(|_| ClipError::Unavailable)?;
        Ok(Self {
            ctx: Mutex::new(ctx),
            poll_interval,
            watcher: Mutex::new(None),
            capability: detect_capability(),
        })
    }
}

/// What this adapter can observe. On Linux the X11 backend under a Wayland session only sees
/// what the compositor mirrors into XWayland (11.3). Wayland-native support is decided in
/// task 3.3; macOS access denial is detected separately (3.4).
fn detect_capability() -> CaptureCapability {
    if cfg!(target_os = "linux") && std::env::var_os("WAYLAND_DISPLAY").is_some() {
        CaptureCapability::LimitedXWayland
    } else {
        CaptureCapability::Full
    }
}

/// Assemble a snapshot from the format list, reading content lazily and only when it is
/// neither concealed nor our own write (1.4, 1.5). Style data is read only when text exists.
pub fn build_snapshot(
    formats: &[String],
    read_text: impl FnOnce() -> Option<String>,
    read_html: impl FnOnce() -> Option<String>,
    read_rtf: impl FnOnce() -> Option<String>,
) -> ClipboardSnapshot {
    let names = formats.iter().map(String::as_str);
    let concealed = has_concealed_format(names.clone());
    let own_marker = has_marker_format(names);
    if concealed || own_marker {
        return ClipboardSnapshot {
            concealed,
            own_marker,
            ..ClipboardSnapshot::default()
        };
    }
    let Some(text) = read_text() else {
        return ClipboardSnapshot::default();
    };
    ClipboardSnapshot {
        html: read_html(),
        rtf: read_rtf(),
        text: Some(text),
        concealed: false,
        own_marker: false,
    }
}

struct Forwarder(Sender<ClipboardEvent>);

impl ClipboardHandler for Forwarder {
    fn on_clipboard_change(&mut self) {
        // A closed receiver means the capture service is gone; nothing to do.
        let _ = self.0.send(ClipboardEvent::Changed);
    }
}

impl ClipboardPort for ClipboardRsAdapter {
    fn start_watch(&self, sender: Sender<ClipboardEvent>) -> Result<(), ClipError> {
        let mut slot = self.watcher.lock().map_err(|_| ClipError::Unavailable)?;
        if slot.is_some() {
            return Err(ClipError::WatchAlreadyStarted);
        }
        let mut watcher = ClipboardWatcherContext::new_with_interval(self.poll_interval)
            .map_err(|_| ClipError::Unavailable)?;
        watcher.add_handler(Forwarder(sender));
        let shutdown = watcher.get_shutdown_channel();
        std::thread::Builder::new()
            .name("clipbuf-clipboard-watch".into())
            .spawn(move || watcher.start_watch())
            .map_err(|_| ClipError::Unavailable)?;
        *slot = Some(shutdown);
        Ok(())
    }

    fn read(&self) -> Result<ClipboardSnapshot, ClipError> {
        let ctx = self.ctx.lock().map_err(|_| ClipError::Read)?;
        let formats = ctx.available_formats().map_err(|_| ClipError::Read)?;
        Ok(build_snapshot(
            &formats,
            || ctx.get_text().ok(),
            || ctx.get_html().ok(),
            || ctx.get_rich_text().ok(),
        ))
    }

    fn write(&self, payload: WritePayload<'_>) -> Result<(), ClipError> {
        let mut contents = vec![ClipboardContent::Text(payload.text.to_string())];
        if let Some(html) = payload.html {
            contents.push(ClipboardContent::Html(html.to_string()));
        }
        if let Some(rtf) = payload.rtf {
            contents.push(ClipboardContent::Rtf(rtf.to_string()));
        }
        contents.push(ClipboardContent::Other(
            marker_format().to_string(),
            b"1".to_vec(),
        ));
        let ctx = self.ctx.lock().map_err(|_| ClipError::Write)?;
        ctx.set(contents).map_err(|_| ClipError::Write)
    }

    fn capability(&self) -> CaptureCapability {
        self.capability
    }
}

impl Drop for ClipboardRsAdapter {
    fn drop(&mut self) {
        // Dropping the shutdown handle stops the watcher thread.
        if let Ok(mut slot) = self.watcher.lock() {
            slot.take();
        }
    }
}
