use serde::{Deserialize, Serialize};

/// Whether the selected clipboard adapter can observe other apps' copies (11.3–11.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureCapability {
    Full,
    /// X11 adapter under a Wayland session: only what the compositor syncs to XWayland.
    LimitedXWayland,
    Unavailable,
    /// macOS pasteboard access denied in Privacy & Security.
    Denied,
}

/// Payload of the `capture-status` event: a capability, or a runtime read failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureStatus {
    Full,
    LimitedXWayland,
    Unavailable,
    Denied,
    ReadFailed,
}

impl From<CaptureCapability> for CaptureStatus {
    fn from(value: CaptureCapability) -> Self {
        match value {
            CaptureCapability::Full => Self::Full,
            CaptureCapability::LimitedXWayland => Self::LimitedXWayland,
            CaptureCapability::Unavailable => Self::Unavailable,
            CaptureCapability::Denied => Self::Denied,
        }
    }
}

/// Result of `get_platform_info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    /// `"macos"`, `"windows"` or `"linux"`.
    pub os: String,
    /// `"quartz"`, `"win32"`, `"x11"`, `"wayland"` or `"none"`.
    pub display_server: String,
    pub capture: CaptureCapability,
}
