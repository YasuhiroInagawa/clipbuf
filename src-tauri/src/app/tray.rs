//! System tray / menu bar icon with "Show / Hide", "Settings", "About", "Quit" (8.4, 8.8).
//!
//! The labels are translated (12.1). They are built from the stored language, and `refresh`
//! rebuilds them when that setting changes, so the tray follows the windows without a restart.

use tauri::AppHandle;
use tauri::Runtime;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

use super::i18n;
use super::window;
use crate::model::Language;

const ID_TOGGLE: &str = "toggle";
const ID_SETTINGS: &str = "settings";
const ID_ABOUT: &str = "about";
const ID_QUIT: &str = "quit";

/// Id of the tray icon, needed to find it again when the language changes.
const TRAY_ID: &str = "clipbuf";

/// The menu in `language`; rebuilt rather than mutated, since menu item text is not editable.
fn build_menu<R: Runtime>(app: &AppHandle<R>, language: Language) -> tauri::Result<Menu<R>> {
    let label = |key| i18n::t(language, key);
    let toggle = MenuItem::with_id(app, ID_TOGGLE, label(i18n::TRAY_TOGGLE), true, None::<&str>)?;
    let settings = MenuItem::with_id(
        app,
        ID_SETTINGS,
        label(i18n::TRAY_SETTINGS),
        true,
        None::<&str>,
    )?;
    let about = MenuItem::with_id(app, ID_ABOUT, label(i18n::TRAY_ABOUT), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ID_QUIT, label(i18n::TRAY_QUIT), true, None::<&str>)?;
    Menu::with_items(app, &[&toggle, &settings, &about, &quit])
}

/// Re-label the tray after the language setting changed (12.4). Missing tray icon is not an
/// error: on some Linux desktops there is none.
pub fn refresh(app: &AppHandle, language: Language) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        match build_menu(app, language) {
            Ok(menu) => {
                if let Err(e) = tray.set_menu(Some(menu)) {
                    log::warn!("tray: could not replace the menu: {e}");
                }
            }
            Err(e) => log::warn!("tray: could not build the menu: {e}"),
        }
    }
}

pub fn install(app: &AppHandle, language: Language) -> tauri::Result<()> {
    let menu = build_menu(app, language)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
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
