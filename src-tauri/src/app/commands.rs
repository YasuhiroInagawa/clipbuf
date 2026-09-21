//! Frontend-facing commands: thin `#[tauri::command]` wrappers over `ops`.
//! Function names are the command names the frontend invokes (design §commands).

use std::sync::Arc;

use tauri::State;

use super::events::{EventSink, TauriSink};
use super::ops;
use super::platform;
use super::state::AppState;
use crate::model::{
    AppError, ItemDto, ItemId, PlatformInfo, Settings, TransferMode, TransferOutcome,
};

pub type SharedState = Arc<AppState>;

#[tauri::command]
pub fn list_items(state: State<'_, SharedState>) -> Vec<ItemDto> {
    ops::list_items(&state)
}

#[tauri::command]
pub fn transfer_item(
    state: State<'_, SharedState>,
    id: ItemId,
    mode: TransferMode,
) -> Result<TransferOutcome, AppError> {
    ops::transfer_item(&state, id, mode)
}

#[tauri::command]
pub fn remove_item(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
    id: ItemId,
) -> Result<(), AppError> {
    ops::remove_item(&state, &TauriSink(app), id)
}

#[tauri::command]
pub fn clear_items(app: tauri::AppHandle, state: State<'_, SharedState>) {
    ops::clear_items(&state, &TauriSink(app));
}

#[tauri::command]
pub fn get_settings(state: State<'_, SharedState>) -> Settings {
    ops::get_settings(&state)
}

/// Validates and persists; applying the diff (hotkey, autostart, capacity) is added by the
/// runtime wiring (task 4.5).
#[tauri::command]
pub fn set_settings(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
    settings: Settings,
) -> Result<Settings, AppError> {
    let (applied, diff) = ops::update_settings(&state, settings)?;
    if diff.any() {
        TauriSink(app).settings_changed(applied.clone());
    }
    Ok(applied)
}

#[tauri::command]
pub fn get_platform_info(state: State<'_, SharedState>) -> PlatformInfo {
    platform::platform_info(state.clipboard.as_ref())
}

#[tauri::command]
pub fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}
