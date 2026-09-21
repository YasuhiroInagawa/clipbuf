//! Launch-at-login (9.5). The plugin is reached through `AutostartControl` so the settings
//! application logic can be tested with a fake.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Opaque failure; details are logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutostartError;

pub trait AutostartControl {
    fn set_enabled(&self, enabled: bool) -> Result<(), AutostartError>;
}

pub struct PluginAutostart(pub AppHandle);

impl AutostartControl for PluginAutostart {
    fn set_enabled(&self, enabled: bool) -> Result<(), AutostartError> {
        let manager = self.0.autolaunch();
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        result.map_err(|e| {
            log::warn!("autostart: could not set enabled={enabled}: {e}");
            AutostartError
        })
    }
}

/// Bring the OS registration in line with the persisted setting (best effort, at startup).
pub fn sync(app: &AppHandle, wanted: bool) {
    let manager = app.autolaunch();
    match manager.is_enabled() {
        Ok(current) if current == wanted => {}
        Ok(_) => {
            let _ = PluginAutostart(app.clone()).set_enabled(wanted);
        }
        Err(e) => log::warn!("autostart: could not query state: {e}"),
    }
}
