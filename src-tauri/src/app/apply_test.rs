use std::str::FromStr;
use std::sync::{Arc, Mutex};

use super::autostart::{AutostartControl, AutostartError};
use super::events::EventSink;
use super::hotkey::{HotkeyState, Registrar, RegistrarError};
use super::ops::apply_settings;
use super::state::AppState;
use super::{RuntimeOpts, parse_runtime_opts};
use crate::buffer::PushResult;
use crate::clipboard::fake::FakeClipboard;
use crate::model::{
    CaptureCapability, CaptureStatus, ErrorKind, ItemDto, Settings, TransferOptions,
};
use crate::settings::SettingsStore;
use tauri_plugin_global_shortcut::Shortcut;

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

struct FakeRegistrar {
    reject: Vec<Shortcut>,
    registered: Mutex<Vec<Shortcut>>,
}

impl FakeRegistrar {
    fn new(reject: &[&str]) -> Self {
        Self {
            reject: reject
                .iter()
                .map(|s| Shortcut::from_str(s).unwrap())
                .collect(),
            registered: Mutex::new(Vec::new()),
        }
    }
}

impl Registrar for FakeRegistrar {
    fn register(&self, shortcut: &Shortcut) -> Result<(), RegistrarError> {
        if self.reject.contains(shortcut) {
            return Err(RegistrarError);
        }
        self.registered.lock().unwrap().push(*shortcut);
        Ok(())
    }
    fn unregister(&self, shortcut: &Shortcut) -> Result<(), RegistrarError> {
        self.registered.lock().unwrap().retain(|r| r != shortcut);
        Ok(())
    }
}

#[derive(Default)]
struct FakeAutostart {
    enabled: Mutex<bool>,
    fail: bool,
}

impl AutostartControl for FakeAutostart {
    fn set_enabled(&self, enabled: bool) -> Result<(), AutostartError> {
        if self.fail {
            return Err(AutostartError);
        }
        *self.enabled.lock().unwrap() = enabled;
        Ok(())
    }
}

struct Fixture {
    state: AppState,
    hotkeys: HotkeyState,
    registrar: FakeRegistrar,
    autostart: FakeAutostart,
    sink: RecordingSink,
}

fn fixture(reject: &[&str]) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::open(dir.path().join("settings.json")).unwrap();
    std::mem::forget(dir);
    let state = AppState::new(store, Arc::new(FakeClipboard::new(CaptureCapability::Full)));
    let hotkeys = HotkeyState::default();
    let registrar = FakeRegistrar::new(reject);
    hotkeys
        .apply(&registrar, &state.settings.get().hotkey)
        .unwrap();
    Fixture {
        state,
        hotkeys,
        registrar,
        autostart: FakeAutostart::default(),
        sink: RecordingSink::default(),
    }
}

impl Fixture {
    fn apply(&self, next: Settings) -> Result<Settings, crate::model::AppError> {
        apply_settings(
            &self.state,
            &self.hotkeys,
            &self.registrar,
            &self.autostart,
            &self.sink,
            next,
        )
    }
}

fn fill(state: &AppState, n: usize) {
    let mut buf = state.buffer.lock().unwrap();
    for i in 0..n {
        assert!(matches!(
            buf.push(format!("item {i}"), None, None, vec![]),
            PushResult::Pushed(_)
        ));
    }
}

#[test]
fn hotkey_failure_saves_nothing_and_keeps_the_previous_shortcut() {
    let f = fixture(&["Ctrl+Shift+C"]);
    let next = Settings {
        hotkey: "Ctrl+Shift+C".into(),
        capacity: 5,
        ..Settings::default()
    };
    let err = f.apply(next).unwrap_err();
    assert_eq!(err.kind, ErrorKind::HotkeyUnavailable);
    assert_eq!(
        f.state.settings.get(),
        Settings::default(),
        "nothing persisted"
    );
    assert_eq!(
        f.hotkeys.current(),
        Some(Shortcut::from_str("Alt+Shift+V").unwrap())
    );
    assert_eq!(
        f.state.buffer.lock().unwrap().capacity(),
        20,
        "capacity not applied"
    );
    let rec = f.sink.0.lock().unwrap();
    assert!(rec.settings_changed.is_empty() && rec.items_changed.is_empty());
}

#[test]
fn invalid_settings_are_rejected_before_touching_the_hotkey() {
    let f = fixture(&[]);
    let next = Settings {
        hotkey: "Ctrl+Shift+C".into(),
        capacity: 0,
        ..Settings::default()
    };
    assert_eq!(f.apply(next).unwrap_err().kind, ErrorKind::InvalidSettings);
    assert_eq!(
        f.hotkeys.current(),
        Some(Shortcut::from_str("Alt+Shift+V").unwrap())
    );
}

#[test]
fn successful_update_applies_every_diff_and_announces() {
    let f = fixture(&[]);
    fill(&f.state, 10);
    let next = Settings {
        hotkey: "Ctrl+Shift+C".into(),
        capacity: 3,
        autostart: true,
        transfer: TransferOptions {
            trim: true,
            ..TransferOptions::default()
        },
        ..Settings::default()
    };
    let applied = f.apply(next.clone()).unwrap();
    assert_eq!(applied, next);
    assert_eq!(f.state.settings.get(), next, "persisted");
    assert_eq!(
        f.hotkeys.current(),
        Some(Shortcut::from_str("Ctrl+Shift+C").unwrap())
    );
    assert_eq!(
        f.state.buffer.lock().unwrap().len(),
        3,
        "buffer truncated (2.5)"
    );
    assert!(*f.autostart.enabled.lock().unwrap());
    let rec = f.sink.0.lock().unwrap();
    assert_eq!(rec.items_changed.len(), 1);
    assert_eq!(rec.items_changed[0].len(), 3);
    assert_eq!(rec.settings_changed, vec![next]);
}

#[test]
fn unchanged_settings_produce_no_events_and_no_side_effects() {
    let f = fixture(&[]);
    fill(&f.state, 5);
    let applied = f.apply(Settings::default()).unwrap();
    assert_eq!(applied, Settings::default());
    let rec = f.sink.0.lock().unwrap();
    assert!(rec.items_changed.is_empty());
    assert!(rec.settings_changed.is_empty());
    assert_eq!(f.state.buffer.lock().unwrap().len(), 5);
}

#[test]
fn growing_capacity_does_not_announce_the_list() {
    let f = fixture(&[]);
    fill(&f.state, 2);
    let next = Settings {
        capacity: 50,
        ..Settings::default()
    };
    f.apply(next).unwrap();
    let rec = f.sink.0.lock().unwrap();
    assert!(rec.items_changed.is_empty(), "no items were dropped");
    assert_eq!(rec.settings_changed.len(), 1);
}

#[test]
fn autostart_failure_is_logged_but_does_not_fail_the_update() {
    let mut f = fixture(&[]);
    f.autostart.fail = true;
    let next = Settings {
        autostart: true,
        ..Settings::default()
    };
    let applied = f.apply(next.clone()).unwrap();
    assert_eq!(applied, next);
    assert_eq!(f.state.settings.get(), next);
}

#[test]
fn runtime_opts_detect_the_hidden_flag() {
    let none: [String; 0] = [];
    assert_eq!(
        parse_runtime_opts(none.iter()),
        RuntimeOpts {
            start_hidden: false
        }
    );
    let args = ["/path/clipbuf".to_string()];
    assert_eq!(
        parse_runtime_opts(args.iter()),
        RuntimeOpts {
            start_hidden: false
        }
    );
    let args = ["/path/clipbuf".to_string(), "--hidden".to_string()];
    assert_eq!(
        parse_runtime_opts(args.iter()),
        RuntimeOpts { start_hidden: true }
    );
}
