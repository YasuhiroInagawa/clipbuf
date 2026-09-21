//! macOS pasteboard privacy (requirement 11.6).
//!
//! Since macOS 15.4 the user can set per-app pasteboard access to always allow / ask / deny in
//! System Settings › Privacy & Security. Reading `changeCount` never triggers the prompt; reading
//! content does. We only report an explicit "deny" so the UI can point the user at the setting.

use objc2::runtime::NSObjectProtocol;
use objc2::sel;
use objc2_app_kit::{NSPasteboard, NSPasteboardAccessBehavior};

/// True when the user set this app's pasteboard access to "Always Deny".
/// On macOS versions without `accessBehavior` this is always false.
pub fn is_denied() -> bool {
    let pasteboard = NSPasteboard::generalPasteboard();
    if !pasteboard.respondsToSelector(sel!(accessBehavior)) {
        return false;
    }
    pasteboard.accessBehavior() == NSPasteboardAccessBehavior::AlwaysDeny
}
