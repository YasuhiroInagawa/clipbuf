//! Tauri runtime wiring: state, capture service, commands, events, tray, window, hotkey.

pub mod capture;
pub mod commands;
pub mod events;
pub mod hotkey;
pub mod ops;
pub mod platform;
pub mod state;
pub mod tray;
pub mod window;

#[cfg(test)]
mod capture_test;
#[cfg(test)]
mod commands_test;
#[cfg(test)]
mod hotkey_test;
#[cfg(test)]
mod platform_test;
#[cfg(test)]
mod window_test;

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::clipboard::select_adapter;
use crate::model::{AppError, ErrorKind};
use crate::settings::SettingsStore;
use capture::CaptureService;
use events::{EventSink, TauriSink};
use hotkey::{HotkeyState, PluginRegistrar};
use state::AppState;

/// Coalescing window for bursts of clipboard changes (design §CaptureService).
const CAPTURE_DEBOUNCE: Duration = Duration::from_millis(100);

/// Build the runtime state, start capturing, wire tray / hotkey / window behaviour and report
/// the initial capture status. Single-instance, autostart and settings-diff application are
/// added by task 4.5.
pub fn bootstrap(app: &AppHandle) -> Result<(), AppError> {
    let settings = SettingsStore::load(app)?;
    let clipboard = select_adapter(&settings.get());
    let state = Arc::new(AppState::new(settings, clipboard));
    let sink: Arc<dyn EventSink> = Arc::new(TauriSink(app.clone()));

    let status = state.clipboard.capability().into();
    let service = CaptureService::start(state.clone(), sink.clone(), CAPTURE_DEBOUNCE)
        .map_err(|_| AppError::from(ErrorKind::CaptureUnavailable))?;

    window::apply_activation_policy(app);
    window::reassert_always_on_top(app);
    window::install_close_to_hide(app);
    if let Err(e) = tray::install(app) {
        log::warn!("tray: not installed: {e}");
    }
    let hotkeys = HotkeyState::default();
    let shortcut = state.settings.get().hotkey;
    if hotkeys
        .apply(&PluginRegistrar(app.clone()), &shortcut)
        .is_err()
    {
        // Not fatal: the tray still works and the settings UI lets the user pick another key.
        log::warn!("hotkey: could not register the configured shortcut");
    }

    app.manage(state);
    app.manage(service);
    app.manage(hotkeys);
    sink.capture_status(status);
    Ok(())
}
