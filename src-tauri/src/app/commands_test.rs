use std::sync::{Arc, Mutex};

use super::events::EventSink;
use super::ops::{
    clear_items, get_settings, list_items, preview_transfer, remove_item, transfer_item,
};
use super::state::AppState;
use crate::buffer::PushResult;
use crate::clipboard::fake::FakeClipboard;
use crate::model::{
    CaptureCapability, CaptureStatus, ErrorKind, ItemDto, NewlineMode, Settings, TransferMode,
    TransferOptions, Warning,
};
use crate::settings::SettingsStore;

#[derive(Default)]
struct Recorded {
    items_changed: Vec<Vec<ItemDto>>,
    settings_changed: Vec<Settings>,
}

#[derive(Default)]
struct RecordingSink(Mutex<Recorded>);

impl EventSink for RecordingSink {
    fn item_added(&self, _item: ItemDto) {}
    fn items_changed(&self, items: Vec<ItemDto>) {
        self.0.lock().unwrap().items_changed.push(items);
    }
    fn settings_changed(&self, settings: Settings) {
        self.0.lock().unwrap().settings_changed.push(settings);
    }
    fn window_shown(&self) {}
    fn capture_status(&self, _status: CaptureStatus) {}
}

struct Fixture {
    fake: Arc<FakeClipboard>,
    state: AppState,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let settings = SettingsStore::open(dir.path().join("settings.json")).unwrap();
    std::mem::forget(dir);
    let fake = Arc::new(FakeClipboard::new(CaptureCapability::Full));
    let state = AppState::new(settings, fake.clone());
    Fixture { fake, state }
}

fn push(state: &AppState, text: &str, html: Option<&str>) -> u64 {
    let mut buf = state.buffer.lock().unwrap();
    let warnings = crate::analysis::analyze(text, html.is_some());
    match buf.push(text.into(), html.map(str::to_string), None, warnings) {
        PushResult::Pushed(id) => id,
        PushResult::Duplicate => panic!("duplicate"),
    }
}

fn set_transfer(state: &AppState, transfer: TransferOptions) {
    let next = Settings {
        transfer,
        ..state.settings.get()
    };
    state.settings.update(next).unwrap();
}

const STYLED: &str = "\tstyled\r\n";
const HTML: &str = "<b>styled</b>";

// ---- transfer branches -------------------------------------------------------------------

#[test]
fn options_with_keep_style_on_styled_item_writes_raw_and_reports_skipped_transforms() {
    let f = fixture();
    let id = push(&f.state, STYLED, Some(HTML));
    set_transfer(
        &f.state,
        TransferOptions {
            keep_style: true,
            newline: NewlineMode::Remove,
            trim: true,
            ..TransferOptions::default()
        },
    );
    let outcome = transfer_item(&f.state, id, TransferMode::Options).unwrap();
    assert!(outcome.skipped_transforms);
    let w = f.fake.written();
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].text, STYLED);
    assert_eq!(w[0].html.as_deref(), Some(HTML));
    assert!(f.state.last_write.matches(STYLED));
}

#[test]
fn options_with_keep_style_on_plain_item_applies_transforms() {
    let f = fixture();
    let id = push(&f.state, STYLED, None); // no style data despite the tab/newline
    set_transfer(
        &f.state,
        TransferOptions {
            keep_style: true,
            newline: NewlineMode::Remove,
            trim: true,
            ..TransferOptions::default()
        },
    );
    let outcome = transfer_item(&f.state, id, TransferMode::Options).unwrap();
    assert!(!outcome.skipped_transforms);
    let w = f.fake.written();
    assert_eq!(w[0].text, "styled");
    assert_eq!(w[0].html, None);
    assert!(f.state.last_write.matches("styled"));
}

#[test]
fn options_without_keep_style_drops_style_and_applies_transforms() {
    let f = fixture();
    let id = push(&f.state, STYLED, Some(HTML));
    set_transfer(
        &f.state,
        TransferOptions {
            keep_style: false,
            newline: NewlineMode::Space,
            tabs_to_spaces: true,
            ..TransferOptions::default()
        },
    );
    let outcome = transfer_item(&f.state, id, TransferMode::Options).unwrap();
    assert!(!outcome.skipped_transforms);
    let w = f.fake.written();
    assert_eq!(w[0].text, "    styled "); // tab → 4 spaces (default width), CRLF → 1 space
    assert_eq!(w[0].html, None);
}

#[test]
fn options_uses_the_configured_tab_width() {
    let f = fixture();
    let id = push(&f.state, "a\tb", None);
    let next = Settings {
        tab_width: 2,
        transfer: TransferOptions {
            tabs_to_spaces: true,
            ..TransferOptions::default()
        },
        ..f.state.settings.get()
    };
    f.state.settings.update(next).unwrap();
    transfer_item(&f.state, id, TransferMode::Options).unwrap();
    assert_eq!(f.fake.written()[0].text, "a  b");
}

#[test]
fn plain_mode_writes_text_only_regardless_of_options() {
    let f = fixture();
    let id = push(&f.state, STYLED, Some(HTML));
    set_transfer(
        &f.state,
        TransferOptions {
            keep_style: true,
            newline: NewlineMode::Remove,
            ..TransferOptions::default()
        },
    );
    let outcome = transfer_item(&f.state, id, TransferMode::Plain).unwrap();
    assert!(!outcome.skipped_transforms);
    let w = f.fake.written();
    assert_eq!(w[0].text, STYLED);
    assert_eq!(w[0].html, None);
    assert_eq!(w[0].rtf, None);
}

#[test]
fn raw_mode_writes_text_and_style_regardless_of_options() {
    let f = fixture();
    let id = push(&f.state, STYLED, Some(HTML));
    set_transfer(
        &f.state,
        TransferOptions {
            keep_style: false,
            newline: NewlineMode::Remove,
            trim: true,
            ..TransferOptions::default()
        },
    );
    let outcome = transfer_item(&f.state, id, TransferMode::Raw).unwrap();
    assert!(!outcome.skipped_transforms);
    let w = f.fake.written();
    assert_eq!(w[0].text, STYLED);
    assert_eq!(w[0].html.as_deref(), Some(HTML));
}

#[test]
fn transfer_never_mutates_the_buffered_item() {
    let f = fixture();
    let id = push(&f.state, STYLED, Some(HTML));
    set_transfer(
        &f.state,
        TransferOptions {
            newline: NewlineMode::Remove,
            trim: true,
            tabs_to_spaces: true,
            ..TransferOptions::default()
        },
    );
    let before = list_items(&f.state);
    for mode in [
        TransferMode::Options,
        TransferMode::Plain,
        TransferMode::Raw,
    ] {
        transfer_item(&f.state, id, mode).unwrap();
    }
    assert_eq!(list_items(&f.state), before);
    let buf = f.state.buffer.lock().unwrap();
    let item = buf.get(id).unwrap();
    assert_eq!(item.text, STYLED);
    assert_eq!(item.html.as_deref(), Some(HTML));
    assert_eq!(
        item.warnings,
        vec![Warning::HasStyle, Warning::EdgeWhitespace, Warning::HasTab]
    );
}

#[test]
fn transfer_errors_carry_only_a_kind() {
    let f = fixture();
    assert_eq!(
        transfer_item(&f.state, 42, TransferMode::Plain)
            .unwrap_err()
            .kind,
        ErrorKind::ItemNotFound
    );
    let id = push(&f.state, "x", None);
    f.fake.fail_writes(true);
    let err = transfer_item(&f.state, id, TransferMode::Plain).unwrap_err();
    assert_eq!(err.kind, ErrorKind::WriteFailed);
    assert_eq!(
        serde_json::to_string(&err).unwrap(),
        r#"{"kind":"writeFailed"}"#
    );
    // A failed write must not poison self-write exclusion.
    assert!(!f.state.last_write.matches("x"));
}

// ---- list / remove / clear ----------------------------------------------------------------

#[test]
fn list_items_is_newest_first_as_dtos() {
    let f = fixture();
    push(&f.state, "one", None);
    push(&f.state, "two", Some("<i>two</i>"));
    let items = list_items(&f.state);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].text, "two");
    assert!(items[0].has_style);
    assert_eq!(items[1].text, "one");
    assert!(!items[1].has_style);
}

#[test]
fn remove_item_deletes_one_and_announces_the_new_list() {
    let f = fixture();
    let sink = RecordingSink::default();
    let a = push(&f.state, "a", None);
    let b = push(&f.state, "b", None);
    remove_item(&f.state, &sink, a).unwrap();
    let rec = sink.0.lock().unwrap();
    assert_eq!(rec.items_changed.len(), 1);
    assert_eq!(
        rec.items_changed[0]
            .iter()
            .map(|i| i.id)
            .collect::<Vec<_>>(),
        vec![b]
    );
    drop(rec);
    assert_eq!(
        remove_item(&f.state, &sink, a).unwrap_err().kind,
        ErrorKind::ItemNotFound
    );
    assert_eq!(
        sink.0.lock().unwrap().items_changed.len(),
        1,
        "no event on failure"
    );
}

#[test]
fn clear_items_empties_the_buffer_and_announces() {
    let f = fixture();
    let sink = RecordingSink::default();
    push(&f.state, "a", None);
    push(&f.state, "b", None);
    clear_items(&f.state, &sink);
    assert!(list_items(&f.state).is_empty());
    let rec = sink.0.lock().unwrap();
    assert_eq!(rec.items_changed, vec![Vec::<ItemDto>::new()]);
}

// ---- settings ----------------------------------------------------------------------------

#[test]
fn get_settings_reflects_the_store() {
    let f = fixture();
    assert_eq!(get_settings(&f.state), Settings::default());
    let next = Settings {
        capacity: 3,
        ..Settings::default()
    };
    f.state.settings.update(next.clone()).unwrap();
    assert_eq!(get_settings(&f.state), next);
}

// ---- preview ------------------------------------------------------------------------------

#[test]
fn preview_matches_what_transfer_would_write_for_every_option_set() {
    for (options, tab_width) in [
        (TransferOptions::default(), 4),
        (
            TransferOptions {
                newline: NewlineMode::Space,
                trim: true,
                tabs_to_spaces: true,
                fullwidth_to_space: true,
                keep_style: false,
            },
            2,
        ),
        (
            TransferOptions {
                keep_style: true,
                newline: NewlineMode::Remove,
                ..TransferOptions::default()
            },
            4,
        ),
    ] {
        // plain item
        let f = fixture();
        let id = push(&f.state, " a\tb\r\n\u{3000}c ", None);
        let next = Settings {
            tab_width,
            transfer: options,
            ..f.state.settings.get()
        };
        f.state.settings.update(next).unwrap();
        let preview = preview_transfer(&f.state, id).unwrap();
        let outcome = transfer_item(&f.state, id, TransferMode::Options).unwrap();
        assert_eq!(preview.text, f.fake.written()[0].text, "{options:?}");
        assert_eq!(preview.skipped_transforms, outcome.skipped_transforms);

        // styled item: keep_style + transforms must report the same skip decision
        let g = fixture();
        let sid = push(&g.state, " a\tb\r\n\u{3000}c ", Some(HTML));
        let next = Settings {
            tab_width,
            transfer: options,
            ..g.state.settings.get()
        };
        g.state.settings.update(next).unwrap();
        let preview = preview_transfer(&g.state, sid).unwrap();
        let outcome = transfer_item(&g.state, sid, TransferMode::Options).unwrap();
        assert_eq!(preview.text, g.fake.written()[0].text, "styled {options:?}");
        assert_eq!(preview.skipped_transforms, outcome.skipped_transforms);
    }
}

#[test]
fn preview_does_not_touch_the_clipboard_or_the_buffer() {
    let f = fixture();
    let id = push(&f.state, "text\n", None);
    let before = list_items(&f.state);
    preview_transfer(&f.state, id).unwrap();
    assert!(f.fake.written().is_empty(), "preview must not write");
    assert!(
        !f.state.last_write.matches("text\n"),
        "preview must not record a write"
    );
    assert_eq!(list_items(&f.state), before);
}

#[test]
fn preview_of_an_unknown_item_reports_item_not_found() {
    let f = fixture();
    assert_eq!(
        preview_transfer(&f.state, 999).unwrap_err().kind,
        ErrorKind::ItemNotFound
    );
}
