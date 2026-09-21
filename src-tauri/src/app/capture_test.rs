use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::capture::{CaptureService, READ_FAILURE_THRESHOLD, process_once};
use super::events::EventSink;
use super::state::AppState;
use crate::clipboard::fake::FakeClipboard;
use crate::model::{
    CaptureCapability, CaptureStatus, ClipboardSnapshot, ItemDto, Settings, Warning,
};
use crate::settings::SettingsStore;

#[derive(Debug, PartialEq, Eq)]
enum Sunk {
    Item(ItemDto),
    Status(CaptureStatus),
}

struct RecordingSink(Mutex<Sender<Sunk>>);

impl EventSink for RecordingSink {
    fn item_added(&self, item: ItemDto) {
        let _ = self.0.lock().unwrap().send(Sunk::Item(item));
    }
    fn items_changed(&self, _items: Vec<ItemDto>) {}
    fn settings_changed(&self, _settings: Settings) {}
    fn window_shown(&self) {}
    fn capture_status(&self, status: CaptureStatus) {
        let _ = self.0.lock().unwrap().send(Sunk::Status(status));
    }
}

fn sink() -> (Arc<RecordingSink>, Receiver<Sunk>) {
    let (tx, rx) = mpsc::channel();
    (Arc::new(RecordingSink(Mutex::new(tx))), rx)
}

fn state_with(fake: Arc<FakeClipboard>) -> Arc<AppState> {
    let dir = tempfile::tempdir().unwrap();
    let settings = SettingsStore::open(dir.path().join("settings.json")).unwrap();
    std::mem::forget(dir);
    Arc::new(AppState::new(settings, fake))
}

fn text(t: &str) -> ClipboardSnapshot {
    ClipboardSnapshot {
        text: Some(t.to_string()),
        ..ClipboardSnapshot::default()
    }
}

fn no_events(rx: &Receiver<Sunk>) {
    assert!(
        rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "unexpected sink event"
    );
}

// ---- process_once (no thread) ----------------------------------------------------------------

#[test]
fn plain_text_is_captured_analyzed_and_announced_once() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();

    fake.push_change(ClipboardSnapshot {
        html: Some("<b>x</b>".into()),
        ..text("hello\tworld ")
    });
    process_once(&state, sink.as_ref());

    let Sunk::Item(item) = rx.recv_timeout(Duration::from_millis(200)).unwrap() else {
        panic!("expected an item")
    };
    assert_eq!(item.text, "hello\tworld ");
    assert!(item.has_style);
    assert_eq!(
        item.warnings,
        vec![Warning::HasStyle, Warning::EdgeWhitespace, Warning::HasTab]
    );
    no_events(&rx);
    assert_eq!(state.buffer.lock().unwrap().len(), 1);
}

#[test]
fn snapshot_without_text_is_dropped() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    fake.push_change(ClipboardSnapshot {
        html: Some("<img>".into()),
        ..ClipboardSnapshot::default()
    });
    process_once(&state, sink.as_ref());
    no_events(&rx);
    assert!(state.buffer.lock().unwrap().is_empty());
}

#[test]
fn concealed_snapshot_is_dropped() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    fake.push_change(ClipboardSnapshot {
        concealed: true,
        ..text("hunter2")
    });
    process_once(&state, sink.as_ref());
    no_events(&rx);
    assert!(state.buffer.lock().unwrap().is_empty());
}

#[test]
fn own_marker_snapshot_is_dropped() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    fake.push_change(ClipboardSnapshot {
        own_marker: true,
        ..text("ours")
    });
    process_once(&state, sink.as_ref());
    no_events(&rx);
    assert!(state.buffer.lock().unwrap().is_empty());
}

#[test]
fn text_matching_the_last_write_hash_is_dropped() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    state.last_write.record("we wrote this");
    // Marker got lost on the way (some environments drop private formats) but the hash matches.
    fake.push_change(text("we wrote this"));
    process_once(&state, sink.as_ref());
    no_events(&rx);
    assert!(state.buffer.lock().unwrap().is_empty());

    // Different text is still captured.
    fake.push_change(text("someone else"));
    process_once(&state, sink.as_ref());
    assert!(matches!(
        rx.recv_timeout(Duration::from_millis(200)),
        Ok(Sunk::Item(_))
    ));
}

#[test]
fn duplicate_of_newest_item_is_not_announced_again() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    fake.push_change(text("same"));
    process_once(&state, sink.as_ref());
    assert!(matches!(
        rx.recv_timeout(Duration::from_millis(200)),
        Ok(Sunk::Item(_))
    ));
    process_once(&state, sink.as_ref());
    no_events(&rx);
    assert_eq!(state.buffer.lock().unwrap().len(), 1);
}

#[test]
fn read_failures_are_ignored_until_the_threshold_then_reported_once() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    fake.fail_reads(true);
    for _ in 0..READ_FAILURE_THRESHOLD - 1 {
        process_once(&state, sink.as_ref());
        no_events(&rx);
    }
    process_once(&state, sink.as_ref());
    assert_eq!(
        rx.recv_timeout(Duration::from_millis(200)).unwrap(),
        Sunk::Status(CaptureStatus::ReadFailed)
    );
    // Further failures do not repeat the notice; a success resets the counter.
    process_once(&state, sink.as_ref());
    no_events(&rx);
    fake.fail_reads(false);
    fake.push_change(text("back"));
    process_once(&state, sink.as_ref());
    assert!(matches!(
        rx.recv_timeout(Duration::from_millis(200)),
        Ok(Sunk::Item(_))
    ));
    assert_eq!(state.consecutive_read_failures(), 0);
}

// ---- threaded service ------------------------------------------------------------------------

#[test]
fn service_thread_debounces_bursts_and_captures_the_final_content() {
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    let _service = CaptureService::start(state.clone(), sink.clone(), Duration::from_millis(50))
        .expect("service starts");

    // Three rapid changes: only the last one should be read and announced.
    fake.push_change(text("first"));
    fake.push_change(text("second"));
    fake.push_change(text("third"));

    let Sunk::Item(item) = rx.recv_timeout(Duration::from_secs(1)).unwrap() else {
        panic!("expected an item")
    };
    assert_eq!(item.text, "third");
    no_events(&rx);
    assert_eq!(state.buffer.lock().unwrap().len(), 1);

    // Stopping is idempotent and the watcher cannot be started twice on the same clipboard.
    assert!(CaptureService::start(state.clone(), sink.clone(), Duration::from_millis(50)).is_err());
}

#[test]
fn service_keeps_capturing_regardless_of_window_visibility() {
    // The service has no notion of a window at all: repeated changes keep arriving.
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = state_with(fake.clone());
    let (sink, rx) = sink();
    let _service =
        CaptureService::start(state.clone(), sink, Duration::from_millis(20)).expect("starts");
    for t in ["a", "b", "c"] {
        fake.push_change(text(t));
        assert!(
            matches!(rx.recv_timeout(Duration::from_secs(1)), Ok(Sunk::Item(_))),
            "{t}"
        );
    }
    assert_eq!(state.buffer.lock().unwrap().len(), 3);
}
