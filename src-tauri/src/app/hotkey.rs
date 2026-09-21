//! Global hotkey that toggles the main window (8.2, 9.4).
//!
//! `HotkeyState` owns the currently registered shortcut and implements the replace-or-restore
//! rule: unregister the old one, try the new one, and if that fails put the old one back and
//! report `HotkeyUnavailable`. The plugin is reached through the small `Registrar` trait so the
//! rule is unit-tested with a fake.

use std::str::FromStr;
use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::window;
use crate::model::{AppError, ErrorKind};

/// Opaque failure from the OS / plugin; details are logged, never surfaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistrarError;

/// Minimal registration interface over the global-shortcut plugin.
pub trait Registrar {
    fn register(&self, shortcut: &Shortcut) -> Result<(), RegistrarError>;
    fn unregister(&self, shortcut: &Shortcut) -> Result<(), RegistrarError>;
}

#[derive(Debug, Default)]
pub struct HotkeyState {
    current: Mutex<Option<Shortcut>>,
}

impl HotkeyState {
    pub fn current(&self) -> Option<Shortcut> {
        *self.current.lock().expect("hotkey lock")
    }

    /// Make `shortcut` the active hotkey. On any failure the previously active hotkey stays
    /// (or is restored) and `HotkeyUnavailable` is returned.
    pub fn apply(&self, registrar: &dyn Registrar, shortcut: &str) -> Result<(), AppError> {
        let next = Shortcut::from_str(shortcut).map_err(|_| ErrorKind::HotkeyUnavailable)?;
        let mut current = self.current.lock().expect("hotkey lock");
        if *current == Some(next) {
            return Ok(());
        }
        if let Some(previous) = *current {
            let _ = registrar.unregister(&previous);
        }
        if registrar.register(&next).is_ok() {
            *current = Some(next);
            return Ok(());
        }
        if let Some(previous) = *current
            && registrar.register(&previous).is_err()
        {
            log::warn!("hotkey: could not restore the previous shortcut");
            *current = None;
        }
        Err(ErrorKind::HotkeyUnavailable.into())
    }
}

/// `Registrar` over the real plugin; registered shortcuts toggle the main window.
pub struct PluginRegistrar(pub AppHandle);

impl Registrar for PluginRegistrar {
    fn register(&self, shortcut: &Shortcut) -> Result<(), RegistrarError> {
        self.0
            .global_shortcut()
            .on_shortcut(*shortcut, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    window::toggle(app);
                }
            })
            .map_err(|e| {
                log::warn!("hotkey: register failed: {e}");
                RegistrarError
            })
    }

    fn unregister(&self, shortcut: &Shortcut) -> Result<(), RegistrarError> {
        self.0
            .global_shortcut()
            .unregister(*shortcut)
            .map_err(|_| RegistrarError)
    }
}
