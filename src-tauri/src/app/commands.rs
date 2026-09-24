//! Frontend-facing commands: thin `#[tauri::command]` wrappers over `ops`.
//! Function names are the command names the frontend invokes (design §commands).

use std::sync::Arc;

use tauri::State;

use super::autostart::PluginAutostart;
use super::events::TauriSink;
use super::hotkey::{HotkeyState, PluginRegistrar};
use super::ops;
use super::platform;
use super::state::AppState;
use crate::model::{
    AppError, ItemDto, ItemId, PlatformInfo, Settings, TransferMode, TransferOutcome,
    TransferPreview,
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
pub fn preview_transfer(
    state: State<'_, SharedState>,
    id: ItemId,
) -> Result<TransferPreview, AppError> {
    ops::preview_transfer(&state, id)
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

/// Validate, register a changed hotkey, persist, apply the diff and announce (9.3, 9.4).
#[tauri::command]
pub fn set_settings(
    app: tauri::AppHandle,
    state: State<'_, SharedState>,
    hotkeys: State<'_, HotkeyState>,
    settings: Settings,
) -> Result<Settings, AppError> {
    ops::apply_settings(
        &state,
        &hotkeys,
        &PluginRegistrar(app.clone()),
        &PluginAutostart(app.clone()),
        &TauriSink(app),
        settings,
    )
}

#[tauri::command]
pub fn get_platform_info(state: State<'_, SharedState>) -> PlatformInfo {
    platform::platform_info(state.clipboard.as_ref())
}

#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) {
    super::window::hide(&app);
}

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> Result<(), AppError> {
    super::window::open_settings(&app)
        .map_err(|_| AppError::from(crate::model::ErrorKind::SettingsIo))
}
