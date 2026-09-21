use std::str::FromStr;
use std::sync::Mutex;

use super::hotkey::{HotkeyState, Registrar, RegistrarError};
use crate::model::ErrorKind;
use tauri_plugin_global_shortcut::Shortcut;

fn sc(s: &str) -> Shortcut {
    Shortcut::from_str(s).unwrap()
}

/// Fake registrar that refuses the shortcuts in `reject` and records every call.
struct FakeRegistrar {
    reject: Vec<Shortcut>,
    calls: Mutex<Vec<(&'static str, Shortcut)>>,
    registered: Mutex<Vec<Shortcut>>,
}

impl FakeRegistrar {
    fn new(reject: &[&str]) -> Self {
        Self {
            reject: reject.iter().map(|s| sc(s)).collect(),
            calls: Mutex::new(Vec::new()),
            registered: Mutex::new(Vec::new()),
        }
    }
    fn calls(&self) -> Vec<(&'static str, Shortcut)> {
        self.calls.lock().unwrap().clone()
    }
    fn registered(&self) -> Vec<Shortcut> {
        self.registered.lock().unwrap().clone()
    }
}

impl Registrar for FakeRegistrar {
    fn register(&self, shortcut: &Shortcut) -> Result<(), RegistrarError> {
        self.calls.lock().unwrap().push(("register", *shortcut));
        if self.reject.contains(shortcut) {
            return Err(RegistrarError);
        }
        self.registered.lock().unwrap().push(*shortcut);
        Ok(())
    }
    fn unregister(&self, shortcut: &Shortcut) -> Result<(), RegistrarError> {
        self.calls.lock().unwrap().push(("unregister", *shortcut));
        self.registered.lock().unwrap().retain(|r| r != shortcut);
        Ok(())
    }
}

#[test]
fn first_registration_sets_the_current_shortcut() {
    let reg = FakeRegistrar::new(&[]);
    let state = HotkeyState::default();
    state.apply(&reg, "Alt+Shift+V").expect("registers");
    assert_eq!(state.current(), Some(sc("Alt+Shift+V")));
    assert_eq!(reg.registered(), vec![sc("Alt+Shift+V")]);
}

#[test]
fn changing_the_shortcut_unregisters_the_old_one_first() {
    let reg = FakeRegistrar::new(&[]);
    let state = HotkeyState::default();
    state.apply(&reg, "Alt+Shift+V").unwrap();
    state.apply(&reg, "CmdOrCtrl+Shift+C").unwrap();
    assert_eq!(reg.registered(), vec![sc("CmdOrCtrl+Shift+C")]);
    assert_eq!(
        reg.calls(),
        vec![
            ("register", sc("Alt+Shift+V")),
            ("unregister", sc("Alt+Shift+V")),
            ("register", sc("CmdOrCtrl+Shift+C")),
        ]
    );
}

#[test]
fn unparseable_shortcut_is_rejected_and_the_current_one_stays_registered() {
    let reg = FakeRegistrar::new(&[]);
    let state = HotkeyState::default();
    state.apply(&reg, "Alt+Shift+V").unwrap();
    let err = state.apply(&reg, "not a shortcut").unwrap_err();
    assert_eq!(err.kind, ErrorKind::HotkeyUnavailable);
    assert_eq!(reg.registered(), vec![sc("Alt+Shift+V")]);
    assert_eq!(
        reg.calls().len(),
        1,
        "nothing was touched for an unparseable string"
    );
}

#[test]
fn registration_failure_restores_the_previous_shortcut() {
    let reg = FakeRegistrar::new(&["Ctrl+Shift+C"]);
    let state = HotkeyState::default();
    state.apply(&reg, "Alt+Shift+V").unwrap();
    let err = state.apply(&reg, "Ctrl+Shift+C").unwrap_err();
    assert_eq!(err.kind, ErrorKind::HotkeyUnavailable);
    assert_eq!(state.current(), Some(sc("Alt+Shift+V")));
    assert_eq!(
        reg.registered(),
        vec![sc("Alt+Shift+V")],
        "previous shortcut re-registered"
    );
}

#[test]
fn registration_failure_with_no_previous_shortcut_leaves_nothing_registered() {
    let reg = FakeRegistrar::new(&["Alt+Shift+V"]);
    let state = HotkeyState::default();
    let err = state.apply(&reg, "Alt+Shift+V").unwrap_err();
    assert_eq!(err.kind, ErrorKind::HotkeyUnavailable);
    assert!(state.current().is_none());
    assert!(reg.registered().is_empty());
}

#[test]
fn applying_the_same_shortcut_again_is_a_no_op() {
    let reg = FakeRegistrar::new(&[]);
    let state = HotkeyState::default();
    state.apply(&reg, "Alt+Shift+V").unwrap();
    state.apply(&reg, "alt+shift+v").unwrap();
    assert_eq!(reg.calls().len(), 1);
}
