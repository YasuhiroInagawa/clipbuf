use std::sync::mpsc;

use super::conceal::{CONCEALED_FORMATS, has_concealed_format};
use super::fake::FakeClipboard;
use super::marker::{
    LastWrite, MARKER_FORMAT_MACOS, MARKER_FORMAT_MIME, MARKER_FORMAT_WINDOWS, has_marker_format,
    marker_format,
};
use super::*;
use crate::model::{CaptureCapability, ClipboardSnapshot};

fn text_snapshot(text: &str) -> ClipboardSnapshot {
    ClipboardSnapshot {
        text: Some(text.to_string()),
        ..ClipboardSnapshot::default()
    }
}

#[test]
fn marker_format_names_are_fixed_per_platform() {
    assert_eq!(MARKER_FORMAT_MACOS, "org.clipbuf.marker");
    assert_eq!(MARKER_FORMAT_WINDOWS, "clipbuf-marker");
    assert_eq!(MARKER_FORMAT_MIME, "application/x-clipbuf-marker");
    let current = marker_format();
    assert!(
        [
            MARKER_FORMAT_MACOS,
            MARKER_FORMAT_WINDOWS,
            MARKER_FORMAT_MIME
        ]
        .contains(&current)
    );
}

#[test]
fn marker_format_is_recognised_on_any_platform_spelling() {
    let formats = [
        "public.utf8-plain-text".to_string(),
        "org.clipbuf.marker".to_string(),
    ];
    assert!(has_marker_format(formats.iter().map(String::as_str)));
    assert!(has_marker_format(
        ["CF_UNICODETEXT", "clipbuf-marker"].into_iter()
    ));
    assert!(has_marker_format(
        ["text/plain", "application/x-clipbuf-marker"].into_iter()
    ));
    assert!(!has_marker_format(["text/plain", "text/html"].into_iter()));
    assert!(!has_marker_format(std::iter::empty::<&str>()));
}

#[test]
fn concealed_formats_cover_macos_windows_and_kde() {
    assert_eq!(
        CONCEALED_FORMATS,
        [
            "org.nspasteboard.ConcealedType",
            "ExcludeClipboardContentFromMonitorProcessing",
            "x-kde-passwordManagerHint",
        ]
    );
    for f in CONCEALED_FORMATS {
        assert!(has_concealed_format(["text/plain", f].into_iter()), "{f}");
    }
    assert!(!has_concealed_format(
        ["text/plain", "text/html"].into_iter()
    ));
}

#[test]
fn last_write_matches_only_the_recorded_text() {
    let last = LastWrite::default();
    assert!(!last.matches("anything"));
    last.record("hello");
    assert!(last.matches("hello"));
    assert!(!last.matches("hello "));
    last.record("other");
    assert!(!last.matches("hello"));
    assert!(last.matches("other"));
    last.clear();
    assert!(!last.matches("other"));
}

#[test]
fn fake_delivers_changes_and_serves_the_injected_snapshot() {
    let fake = FakeClipboard::new(CaptureCapability::Full);
    let (tx, rx) = mpsc::channel();
    fake.start_watch(tx).expect("first watch starts");

    fake.push_change(text_snapshot("from another app"));
    assert_eq!(rx.try_recv(), Ok(ClipboardEvent::Changed));
    let snapshot = fake.read().expect("readable");
    assert_eq!(snapshot.text.as_deref(), Some("from another app"));
    assert!(!snapshot.concealed);
    assert!(!snapshot.own_marker);
    assert_eq!(fake.capability(), CaptureCapability::Full);
}

#[test]
fn fake_rejects_a_second_watcher() {
    let fake = FakeClipboard::new(CaptureCapability::Full);
    let (tx1, _rx1) = mpsc::channel();
    let (tx2, _rx2) = mpsc::channel();
    assert!(fake.start_watch(tx1).is_ok());
    assert_eq!(fake.start_watch(tx2), Err(ClipError::WatchAlreadyStarted));
}

#[test]
fn fake_identifies_concealed_and_marker_snapshots() {
    let fake = FakeClipboard::new(CaptureCapability::Full);
    fake.push_change(ClipboardSnapshot {
        concealed: true,
        ..text_snapshot("hunter2")
    });
    assert!(fake.read().unwrap().concealed);

    fake.push_change(ClipboardSnapshot {
        own_marker: true,
        ..text_snapshot("ours")
    });
    let s = fake.read().unwrap();
    assert!(s.own_marker);
    assert!(!s.concealed);
}

#[test]
fn fake_records_writes_and_exposes_them_as_our_own_content() {
    let fake = FakeClipboard::new(CaptureCapability::Full);
    let (tx, rx) = mpsc::channel();
    fake.start_watch(tx).unwrap();

    fake.write(WritePayload {
        text: "plain",
        html: Some("<b>plain</b>"),
        rtf: None,
    })
    .expect("write ok");

    let written = fake.written();
    assert_eq!(written.len(), 1);
    assert_eq!(written[0].text, "plain");
    assert_eq!(written[0].html.as_deref(), Some("<b>plain</b>"));
    assert_eq!(written[0].rtf, None);

    // A real clipboard notifies watchers about our own write; the snapshot carries the marker.
    assert_eq!(rx.try_recv(), Ok(ClipboardEvent::Changed));
    let s = fake.read().unwrap();
    assert_eq!(s.text.as_deref(), Some("plain"));
    assert_eq!(s.html.as_deref(), Some("<b>plain</b>"));
    assert!(s.own_marker);
}

#[test]
fn fake_can_simulate_read_and_write_failures() {
    let fake = FakeClipboard::new(CaptureCapability::Unavailable);
    fake.fail_reads(true);
    assert_eq!(fake.read(), Err(ClipError::Read));
    fake.fail_reads(false);
    assert!(fake.read().is_ok());

    fake.fail_writes(true);
    let payload = WritePayload {
        text: "x",
        html: None,
        rtf: None,
    };
    assert_eq!(fake.write(payload), Err(ClipError::Write));
    assert!(fake.written().is_empty());
    assert_eq!(fake.capability(), CaptureCapability::Unavailable);
}

#[test]
fn port_is_object_safe_and_thread_safe() {
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn ClipboardPort>();
    let boxed: Box<dyn ClipboardPort> = Box::new(FakeClipboard::new(CaptureCapability::Full));
    assert_eq!(boxed.capability(), CaptureCapability::Full);
}
