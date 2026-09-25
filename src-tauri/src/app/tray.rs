//! System tray / menu bar icon with "Show / Hide", "Settings", "About", "Quit" (8.4, 8.8).

use tauri::AppHandle;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

use super::window;

const ID_TOGGLE: &str = "toggle";
const ID_SETTINGS: &str = "settings";
const ID_ABOUT: &str = "about";
const ID_QUIT: &str = "quit";

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, ID_TOGGLE, "Show / Hide", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, ID_SETTINGS, "Settings…", true, None::<&str>)?;
    let about = MenuItem::with_id(app, ID_ABOUT, "About clipbuf", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit clipbuf", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &settings, &about, &quit])?;

    let mut builder = TrayIconBuilder::with_id("clipbuf")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("clipbuf")
        .on_menu_event(|app, event| match event.id().as_ref() {
            ID_TOGGLE => window::toggle(app),
            ID_SETTINGS => {
                if let Err(e) = window::open_settings(app) {
                    log::warn!("tray: could not open settings window: {e}");
                }
            }
            ID_ABOUT => {
                if let Err(e) = window::open_about(app) {
                    log::warn!("tray: could not open about window: {e}");
                }
            }
            ID_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::toggle(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
