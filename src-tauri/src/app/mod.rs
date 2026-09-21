//! Tauri runtime wiring: state, capture service, commands, events, tray, window, hotkey.

pub mod capture;
pub mod commands;
pub mod events;
pub mod ops;
pub mod platform;
pub mod state;

#[cfg(test)]
mod capture_test;
#[cfg(test)]
mod commands_test;
#[cfg(test)]
mod platform_test;

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::clipboard::select_adapter;
use crate::model::{AppError, ErrorKind};
use crate::settings::SettingsStore;
use capture::CaptureService;
use events::{EventSink, TauriSink};
use state::AppState;

/// Coalescing window for bursts of clipboard changes (design §CaptureService).
const CAPTURE_DEBOUNCE: Duration = Duration::from_millis(100);

/// Build the runtime state, start capturing and report the initial capture status.
/// Tray, hotkey and window behaviour are wired by later steps (tasks 4.4, 4.5).
pub fn bootstrap(app: &AppHandle) -> Result<(), AppError> {
    let settings = SettingsStore::load(app)?;
    let clipboard = select_adapter(&settings.get());
    let state = Arc::new(AppState::new(settings, clipboard));
    let sink: Arc<dyn EventSink> = Arc::new(TauriSink(app.clone()));

    let status = state.clipboard.capability().into();
    let service = CaptureService::start(state.clone(), sink.clone(), CAPTURE_DEBOUNCE)
        .map_err(|_| AppError::from(ErrorKind::CaptureUnavailable))?;
    app.manage(state);
    app.manage(service);
    sink.capture_status(status);
    Ok(())
}
