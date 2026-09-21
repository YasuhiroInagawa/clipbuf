//! Logic behind the frontend commands, testable without Tauri (see `commands.rs` for the
//! `#[tauri::command]` wrappers).

use super::autostart::AutostartControl;
use super::events::EventSink;
use super::hotkey::{HotkeyState, Registrar};
use super::state::AppState;
use crate::clipboard::WritePayload;
use crate::model::{AppError, ErrorKind, ItemDto, ItemId, Settings, TransferMode, TransferOutcome};
use crate::settings;
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

/// Full settings update (9.3, 9.4, 2.5): validate → register a changed hotkey *before*
/// saving (so a rejected hotkey leaves everything untouched) → persist → apply the diff to
/// the buffer and autostart → announce.
pub fn apply_settings(
    state: &AppState,
    hotkeys: &HotkeyState,
    registrar: &dyn Registrar,
    autostart: &dyn AutostartControl,
    sink: &dyn EventSink,
    next: Settings,
) -> Result<Settings, AppError> {
    settings::validate(&next)?;
    let current = state.settings.get();
    let diff = settings::diff(&current, &next);
    if !diff.any() {
        return Ok(current);
    }
    if diff.hotkey {
        hotkeys.apply(registrar, &next.hotkey)?;
    }
    if let Err(e) = state.settings.update(next) {
        // Persisting failed after the hotkey moved: put the old one back so state and disk agree.
        if diff.hotkey {
            let _ = hotkeys.apply(registrar, &current.hotkey);
        }
        return Err(e);
    }
    let applied = state.settings.get();

    if diff.capacity {
        let announce = {
            let mut buffer = state.buffer.lock().expect("buffer lock");
            let before = buffer.len();
            buffer.set_capacity(applied.capacity);
            buffer.len() != before
        };
        if announce {
            sink.items_changed(list_items(state));
        }
    }
    if diff.autostart {
        // Failure is logged by the control; the preference itself is still saved.
        let _ = autostart.set_enabled(applied.autostart);
    }
    sink.settings_changed(applied.clone());
    Ok(applied)
}
