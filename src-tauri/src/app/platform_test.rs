use super::platform::{display_server_name, platform_info};
use crate::clipboard::fake::FakeClipboard;
use crate::model::CaptureCapability;

#[test]
fn display_server_name_is_derived_from_os_and_session_variables() {
    assert_eq!(display_server_name("macos", false, false), "quartz");
    assert_eq!(display_server_name("windows", false, false), "win32");
    assert_eq!(display_server_name("linux", true, true), "wayland");
    assert_eq!(display_server_name("linux", true, false), "wayland");
    assert_eq!(display_server_name("linux", false, true), "x11");
    assert_eq!(display_server_name("linux", false, false), "none");
}

#[test]
fn platform_info_reports_the_adapter_capability() {
    let port = FakeClipboard::new(CaptureCapability::LimitedXWayland);
    let info = platform_info(&port);
    assert_eq!(info.capture, CaptureCapability::LimitedXWayland);
    assert_eq!(info.os, std::env::consts::OS);
    assert!(["quartz", "win32", "wayland", "x11", "none"].contains(&info.display_server.as_str()));
}
