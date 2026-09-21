//! Names of events emitted from Rust to the frontend, and their payload types.
//!
//! | Event | Payload |
//! |---|---|
//! | [`ITEM_ADDED`] | [`ItemDto`](crate::model::ItemDto) |
//! | [`ITEMS_CHANGED`] | `Vec<`[`ItemDto`](crate::model::ItemDto)`>` |
//! | [`SETTINGS_CHANGED`] | [`Settings`](crate::model::Settings) |
//! | [`WINDOW_SHOWN`] | `()` |
//! | [`CAPTURE_STATUS`] | [`CaptureStatus`](crate::model::CaptureStatus) |

/// A new item was captured and placed at the top of the list (1.1).
pub const ITEM_ADDED: &str = "clipbuf://item-added";
/// The whole list changed (removal, clear, capacity shrink) (2.3–2.5).
pub const ITEMS_CHANGED: &str = "clipbuf://items-changed";
/// Settings were updated successfully (9.3).
pub const SETTINGS_CHANGED: &str = "clipbuf://settings-changed";
/// The main window was just shown by hotkey, tray or single-instance (8.3).
pub const WINDOW_SHOWN: &str = "clipbuf://window-shown";
/// Capture capability at startup or a runtime read failure (11.5, 11.6).
pub const CAPTURE_STATUS: &str = "clipbuf://capture-status";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::contract_test::section;

    #[test]
    fn event_names_match_fixture() {
        let expected = section("events");
        let actual = serde_json::json!({
            "itemAdded": ITEM_ADDED,
            "itemsChanged": ITEMS_CHANGED,
            "settingsChanged": SETTINGS_CHANGED,
            "windowShown": WINDOW_SHOWN,
            "captureStatus": CAPTURE_STATUS,
        });
        assert_eq!(actual, expected);
    }
}
