//! Logic behind the frontend commands, testable without Tauri (see `commands.rs` for the
//! `#[tauri::command]` wrappers).

use super::events::EventSink;
use super::state::AppState;
use crate::clipboard::WritePayload;
use crate::model::{AppError, ErrorKind, ItemDto, ItemId, Settings, TransferMode, TransferOutcome};
use crate::settings::SettingsDiff;
use crate::transform;

/// Items newest first (2.1).
pub fn list_items(state: &AppState) -> Vec<ItemDto> {
    state
        .buffer
        .lock()
        .expect("buffer lock")
        .items()
        .map(|item| item.to_dto())
        .collect()
}

/// Write an item back to the clipboard (6.1, 6.5, 6.6, 7.2, 7.4–7.6).
///
/// The buffered item is never modified (6.9); a new string is produced for the write.
/// The text hash is recorded *after* a successful write so a failed write cannot cause a
/// later genuine copy of the same text to be dropped.
pub fn transfer_item(
    state: &AppState,
    id: ItemId,
    mode: TransferMode,
) -> Result<TransferOutcome, AppError> {
    let settings = state.settings.get();
    let (text, html, rtf, skipped_transforms) = {
        let buffer = state.buffer.lock().expect("buffer lock");
        let item = buffer.get(id).ok_or(ErrorKind::ItemNotFound)?;
        let options = &settings.transfer;
        match mode {
            TransferMode::Raw => (
                item.text.clone(),
                item.html.clone(),
                item.rtf.clone(),
                false,
            ),
            TransferMode::Plain => (item.text.clone(), None, None, false),
            TransferMode::Options => {
                let keep_style = options.keep_style && item.has_style();
                if keep_style && options.has_text_transform() {
                    // Style wins; the transforms are not applied (7.5).
                    (item.text.clone(), item.html.clone(), item.rtf.clone(), true)
                } else {
                    let text = transform::apply(&item.text, options, settings.tab_width);
                    if keep_style {
                        (text, item.html.clone(), item.rtf.clone(), false)
                    } else {
                        (text, None, None, false)
                    }
                }
            }
        }
    };

    state
        .clipboard
        .write(WritePayload {
            text: &text,
            html: html.as_deref(),
            rtf: rtf.as_deref(),
        })
        .map_err(|_| AppError::from(ErrorKind::WriteFailed))?;
    state.last_write.record(&text);
    Ok(TransferOutcome { skipped_transforms })
}

/// Remove one item and announce the new list (2.3).
pub fn remove_item(state: &AppState, sink: &dyn EventSink, id: ItemId) -> Result<(), AppError> {
    let removed = state.buffer.lock().expect("buffer lock").remove(id);
    if !removed {
        return Err(ErrorKind::ItemNotFound.into());
    }
    sink.items_changed(list_items(state));
    Ok(())
}

/// Remove every item and announce the (empty) list (2.4).
pub fn clear_items(state: &AppState, sink: &dyn EventSink) {
    state.buffer.lock().expect("buffer lock").clear();
    sink.items_changed(Vec::new());
}

pub fn get_settings(state: &AppState) -> Settings {
    state.settings.get()
}

/// Validate and persist `next`. Applying the diff (hotkey, autostart, capacity) and
/// announcing the change is the runtime's job (task 4.5).
pub fn update_settings(
    state: &AppState,
    next: Settings,
) -> Result<(Settings, SettingsDiff), AppError> {
    let diff = state.settings.update(next)?;
    Ok((state.settings.get(), diff))
}
