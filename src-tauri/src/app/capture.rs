//! Capture service: the only path from a clipboard change to a buffered item (1.1–1.8).
//!
//! A dedicated thread waits for `ClipboardEvent::Changed`, coalesces bursts, reads the
//! clipboard once, applies the exclusion chain, analyzes the text and pushes it into the
//! buffer. Nothing here knows about windows (1.7) and nothing logs item content (10.5).

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::events::EventSink;
use super::state::AppState;
use crate::analysis;
use crate::buffer::PushResult;
use crate::clipboard::{ClipError, ClipboardEvent};
use crate::model::{CaptureStatus, ClipboardSnapshot};

/// Consecutive read failures after which `CaptureStatus::ReadFailed` is reported once.
pub const READ_FAILURE_THRESHOLD: u32 = 5;

/// Upper bound on how long a burst of changes may postpone a read (keeps 1.8 under load).
const MAX_COALESCE: Duration = Duration::from_millis(500);

/// Result of the exclusion chain for one snapshot.
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Capture {
        text: String,
        html: Option<String>,
        rtf: Option<String>,
    },
    NoText,
    Concealed,
    OwnMarker,
    OwnHash,
}

impl Decision {
    /// Content-free label for logs (10.5).
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Capture { .. } => "capture",
            Self::NoText => "no text",
            Self::Concealed => "concealed",
            Self::OwnMarker => "own marker",
            Self::OwnHash => "own hash",
        }
    }
}

/// Exclusion chain in the order required by the design: no text → concealed → own marker →
/// hash of our last write (1.3, 1.5, 1.4).
pub fn decide(snapshot: ClipboardSnapshot, state: &AppState) -> Decision {
    let Some(text) = snapshot.text else {
        return Decision::NoText;
    };
    if snapshot.concealed {
        return Decision::Concealed;
    }
    if snapshot.own_marker {
        return Decision::OwnMarker;
    }
    if state.last_write.matches(&text) {
        return Decision::OwnHash;
    }
    Decision::Capture {
        text,
        html: snapshot.html,
        rtf: snapshot.rtf,
    }
}

/// Read the clipboard once and, if it passes the exclusion chain, add it to the buffer and
/// announce it. Read failures are counted; the threshold triggers a single status report.
pub fn process_once(state: &AppState, sink: &dyn EventSink) {
    let snapshot = match state.clipboard.read() {
        Ok(s) => s,
        Err(_) => {
            if state.note_read_failure() == READ_FAILURE_THRESHOLD {
                log::warn!("capture: {READ_FAILURE_THRESHOLD} consecutive clipboard read failures");
                sink.capture_status(CaptureStatus::ReadFailed);
            }
            return;
        }
    };
    state.note_read_success();

    let (text, html, rtf) = match decide(snapshot, state) {
        Decision::Capture { text, html, rtf } => (text, html, rtf),
        other => {
            log::debug!("capture: dropped ({})", other.reason());
            return;
        }
    };
    let has_style = html.is_some() || rtf.is_some();
    let warnings = analysis::analyze(&text, has_style);

    let (dto, evicted, items) = {
        let mut buffer = state.buffer.lock().expect("buffer lock");
        let before = buffer.len();
        match buffer.push(text, html, rtf, warnings) {
            PushResult::Pushed(id) => {
                // Adding at the capacity limit drops the oldest item; the frontend copy only
                // learns about that from the whole list (2.2).
                let evicted = buffer.len() == before;
                let items = evicted.then(|| buffer.items().map(|i| i.to_dto()).collect());
                (buffer.get(id).map(|item| item.to_dto()), evicted, items)
            }
            PushResult::Duplicate => (None, false, None),
        }
    };
    if let Some(dto) = dto {
        sink.item_added(dto);
    }
    if evicted && let Some(items) = items {
        sink.items_changed(items);
    }
}

/// Handle to the running capture thread. Dropping it does not stop the OS watcher; the
/// adapter owns that.
pub struct CaptureService {
    _thread: JoinHandle<()>,
}

impl CaptureService {
    /// Start watching `state.clipboard` and processing changes on a dedicated thread.
    /// Changes arriving within `debounce` of each other are coalesced into one read.
    pub fn start(
        state: Arc<AppState>,
        sink: Arc<dyn EventSink>,
        debounce: Duration,
    ) -> Result<Self, ClipError> {
        let (tx, rx) = mpsc::channel();
        state.clipboard.start_watch(tx)?;
        let thread = std::thread::Builder::new()
            .name("clipbuf-capture".into())
            .spawn(move || run(rx, state, sink, debounce))
            .map_err(|_| ClipError::Unavailable)?;
        Ok(Self { _thread: thread })
    }
}

fn run(
    rx: Receiver<ClipboardEvent>,
    state: Arc<AppState>,
    sink: Arc<dyn EventSink>,
    debounce: Duration,
) {
    while rx.recv().is_ok() {
        coalesce(&rx, debounce);
        process_once(&state, sink.as_ref());
    }
    log::debug!("capture: watcher channel closed, thread exiting");
}

/// Wait until the clipboard has been quiet for `debounce`, or `MAX_COALESCE` has elapsed.
fn coalesce(rx: &Receiver<ClipboardEvent>, debounce: Duration) {
    let started = Instant::now();
    loop {
        match rx.recv_timeout(debounce) {
            Ok(_) if started.elapsed() < MAX_COALESCE => continue,
            Ok(_) | Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}
