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

mod clipboard_rs_adapter {
    use super::super::clipboard_rs::{ClipboardRsAdapter, build_snapshot};
    use crate::model::ClipboardSnapshot;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn snapshot_carries_text_html_rtf_when_present() {
        let snap = build_snapshot(
            &s(&["public.utf8-plain-text", "public.html", "public.rtf"]),
            || Some("t".into()),
            || Some("<b>t</b>".into()),
            || Some("{\\rtf1 t}".into()),
        );
        assert_eq!(
            snap,
            ClipboardSnapshot {
                text: Some("t".into()),
                html: Some("<b>t</b>".into()),
                rtf: Some("{\\rtf1 t}".into()),
                concealed: false,
                own_marker: false,
            }
        );
    }

    #[test]
    fn snapshot_without_text_does_not_read_style_data() {
        let snap = build_snapshot(
            &s(&["public.png"]),
            || None,
            || panic!("html must not be read when there is no text"),
            || panic!("rtf must not be read when there is no text"),
        );
        assert_eq!(snap, ClipboardSnapshot::default());
    }

    #[test]
    fn concealed_content_is_flagged_and_never_read() {
        let snap = build_snapshot(
            &s(&["public.utf8-plain-text", "org.nspasteboard.ConcealedType"]),
            || panic!("concealed text must not be read"),
            || panic!(),
            || panic!(),
        );
        assert!(snap.concealed);
        assert_eq!(snap.text, None);
    }

    #[test]
    fn own_marker_is_flagged_and_content_not_read() {
        let snap = build_snapshot(
            &s(&["CF_UNICODETEXT", "clipbuf-marker"]),
            || panic!("our own write must not be read back"),
            || panic!(),
            || panic!(),
        );
        assert!(snap.own_marker);
        assert!(!snap.concealed);
        assert_eq!(snap.text, None);
    }

    #[test]
    fn capability_follows_the_selected_backend() {
        use super::super::clipboard_rs::{Backend, capability_for};
        use crate::model::CaptureCapability;
        assert_eq!(
            capability_for(Backend::Wayland, true),
            CaptureCapability::Full
        );
        assert_eq!(
            capability_for(Backend::X11, true),
            CaptureCapability::LimitedXWayland
        );
        assert_eq!(capability_for(Backend::X11, false), CaptureCapability::Full);
        assert_eq!(
            capability_for(Backend::Native, false),
            CaptureCapability::Full
        );
        assert_eq!(
            capability_for(Backend::Native, true),
            CaptureCapability::Full
        );
    }

    #[test]
    fn adapter_is_a_clipboard_port() {
        fn assert_port<T: super::super::ClipboardPort>() {}
        assert_port::<ClipboardRsAdapter>();
    }

    /// Touches the real pasteboard; run manually on macOS with `cargo test -- --ignored`.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "uses the real macOS pasteboard and pbcopy"]
    fn real_pasteboard_round_trip() {
        use super::super::{ClipboardEvent, ClipboardPort, WritePayload};
        use crate::model::CaptureCapability;
        use std::io::Write;
        use std::sync::mpsc;
        use std::time::{Duration, Instant};

        let adapter = ClipboardRsAdapter::new(Duration::from_millis(200)).expect("adapter");
        assert_eq!(adapter.capability(), CaptureCapability::Full);
        let (tx, rx) = mpsc::channel();
        adapter.start_watch(tx).expect("watch");
        assert_eq!(
            adapter.start_watch(mpsc::channel().0),
            Err(super::super::ClipError::WatchAlreadyStarted)
        );
        std::thread::sleep(Duration::from_millis(300));

        // Our own write: marker must be visible in the following snapshot.
        adapter
            .write(WritePayload {
                text: "clipbuf own",
                html: Some("<b>clipbuf own</b>"),
                rtf: None,
            })
            .expect("write");
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(1)),
            Ok(ClipboardEvent::Changed)
        );
        let snap = adapter.read().expect("read");
        assert!(snap.own_marker, "own marker after our write: {snap:?}");
        while rx.try_recv().is_ok() {}

        // Another process copies: event within 1 s, no marker, text readable.
        let t0 = Instant::now();
        let mut child = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("pbcopy");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"from pbcopy")
            .unwrap();
        child.wait().unwrap();
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(1)),
            Ok(ClipboardEvent::Changed)
        );
        assert!(t0.elapsed() < Duration::from_secs(1));
        let snap = adapter.read().expect("read");
        assert_eq!(snap.text.as_deref(), Some("from pbcopy"));
        assert!(!snap.own_marker);
        assert!(!snap.concealed);
        assert_eq!(snap.html, None);
    }
}

mod selection {
    use super::super::unavailable::UnavailableClipboard;
    use super::super::{ClipError, ClipboardPort, WritePayload, select_adapter};
    use crate::model::{CaptureCapability, Settings};
    use std::sync::mpsc;

    #[test]
    fn unavailable_clipboard_reports_itself_and_fails_every_operation() {
        let port = UnavailableClipboard;
        assert_eq!(port.capability(), CaptureCapability::Unavailable);
        assert_eq!(port.read(), Err(ClipError::Unavailable));
        assert_eq!(
            port.write(WritePayload {
                text: "x",
                html: None,
                rtf: None
            }),
            Err(ClipError::Unavailable)
        );
        // Watching an unavailable clipboard is a no-op that never sends anything.
        let (tx, rx) = mpsc::channel();
        assert_eq!(port.start_watch(tx), Ok(()));
        assert!(rx.try_recv().is_err());
    }

    /// On the development Mac (with a pasteboard) selection yields a working adapter.
    #[cfg(target_os = "macos")]
    #[test]
    fn select_adapter_on_macos_yields_full_capability() {
        let port = select_adapter(&Settings::default());
        assert_eq!(port.capability(), CaptureCapability::Full);
    }

    /// Windows always has a clipboard, including on CI runners.
    #[cfg(target_os = "windows")]
    #[test]
    fn select_adapter_on_windows_yields_full_capability() {
        let port = select_adapter(&Settings::default());
        assert_eq!(port.capability(), CaptureCapability::Full);
    }

    /// Headless Linux (CI) has neither DISPLAY nor WAYLAND_DISPLAY: selection must degrade
    /// to the unavailable adapter instead of panicking.
    #[cfg(target_os = "linux")]
    #[test]
    fn select_adapter_without_display_is_unavailable() {
        if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return; // a desktop session; covered by the manual platform checklist
        }
        let port = select_adapter(&Settings::default());
        assert_eq!(port.capability(), CaptureCapability::Unavailable);
    }
}
