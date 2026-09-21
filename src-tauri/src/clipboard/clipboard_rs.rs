//! `ClipboardPort` adapter over the `clipboard-rs` crate (Windows, macOS, Linux X11 and
//! Wayland).
//!
//! Change detection: Windows uses a clipboard listener, macOS and Wayland poll every
//! `poll_interval` (`changeCount` / data-control offers), X11 uses XFixes — all inside
//! `clipboard-rs`'s watcher, which runs on its own thread and only forwards
//! `ClipboardEvent::Changed`. Content is read on demand.
//!
//! On Linux the crate picks the backend at runtime: Wayland (`wayland` feature, data-control
//! protocol) when `WAYLAND_DISPLAY` is set and the compositor supports it, otherwise X11.

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
        let capability = capability_for(
            backend_of(&ctx),
            std::env::var_os("WAYLAND_DISPLAY").is_some(),
        );
        Ok(Self {
            ctx: Mutex::new(ctx),
            poll_interval,
            watcher: Mutex::new(None),
            capability,
        })
    }
}

/// Which clipboard backend `clipboard-rs` selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Windows or macOS: the platform's single native clipboard.
    Native,
    X11,
    Wayland,
}

#[cfg(target_os = "linux")]
fn backend_of(ctx: &ClipboardContext) -> Backend {
    match ctx {
        ClipboardContext::X11(_) => Backend::X11,
        ClipboardContext::Wayland(_) => Backend::Wayland,
    }
}

#[cfg(not(target_os = "linux"))]
fn backend_of(_ctx: &ClipboardContext) -> Backend {
    Backend::Native
}

/// What the adapter can observe (11.3, 11.4). The X11 backend inside a Wayland session only
/// sees what the compositor mirrors into XWayland. macOS access denial is detected
/// separately (task 3.4).
pub fn capability_for(backend: Backend, wayland_session: bool) -> CaptureCapability {
    match backend {
        Backend::X11 if wayland_session => CaptureCapability::LimitedXWayland,
        Backend::X11 | Backend::Wayland | Backend::Native => CaptureCapability::Full,
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
