//! Platform description handed to the frontend (`get_platform_info`, `capture-status`).

use crate::clipboard::ClipboardPort;
use crate::model::PlatformInfo;

/// Name of the display server for `PlatformInfo.display_server`.
pub fn display_server_name(os: &str, wayland_display: bool, x11_display: bool) -> &'static str {
    match os {
        "macos" => "quartz",
        "windows" => "win32",
        _ if wayland_display => "wayland",
        _ if x11_display => "x11",
        _ => "none",
    }
}

pub fn platform_info(port: &dyn ClipboardPort) -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        display_server: display_server_name(
            std::env::consts::OS,
            std::env::var_os("WAYLAND_DISPLAY").is_some(),
            std::env::var_os("DISPLAY").is_some(),
        )
        .to_string(),
        capture: port.capability(),
    }
}
