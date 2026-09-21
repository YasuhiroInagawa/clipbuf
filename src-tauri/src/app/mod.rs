//! Tauri runtime wiring: state, capture service, commands, events, tray, window, hotkey.

pub mod autostart;
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
mod apply_test;
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

/// Argument passed by the autostart entry so a login launch stays out of the way (9.5).
pub const HIDDEN_FLAG: &str = "--hidden";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeOpts {
    pub start_hidden: bool,
}

/// Parse process arguments (the first one is the executable and is ignored).
pub fn parse_runtime_opts<'a>(args: impl Iterator<Item = &'a String>) -> RuntimeOpts {
    RuntimeOpts {
        start_hidden: args.skip(1).any(|a| a == HIDDEN_FLAG),
    }
}

/// Build the runtime state, start capturing, wire tray / hotkey / window behaviour, sync
/// autostart, show the main window unless started hidden, and report the capture status.
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

    autostart::sync(app, state.settings.get().autostart);

    app.manage(state);
    app.manage(service);
    app.manage(hotkeys);

    let args: Vec<String> = std::env::args().collect();
    if !parse_runtime_opts(args.iter()).start_hidden {
        window::show(app);
    }
    sink.capture_status(status);
    Ok(())
}
