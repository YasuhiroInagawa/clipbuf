//! Main, settings and about windows (8.1–8.5, 8.8, 9.1).
//!
//! The main window is configured in `tauri.conf.json` (always on top, skip taskbar, all
//! workspaces). Closing hides it; the tray and the hotkey toggle it. The settings and about
//! windows are further webviews created on first use.

use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::events::{EventSink, TauriSink};

pub const MAIN: &str = "main";
pub const SETTINGS: &str = "settings";
pub const ABOUT: &str = "about";

/// Windows that belong to the main one: hiding clipbuf means "get out of the way", so they go
/// with it rather than being left floating on their own.
const SECONDARY: [&str; 2] = [SETTINGS, ABOUT];

/// Which secondary windows were hidden along with the main window, so that showing it brings
/// them back. A window the user closed themselves is not in here and stays gone.
static HIDDEN_WITH_MAIN: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN)
}

/// Forget that `label` was hidden with the main window: it is being opened or closed
/// deliberately, so the next `show` must not second-guess that.
fn forget_hidden(label: &str) {
    if let Ok(mut hidden) = HIDDEN_WITH_MAIN.lock() {
        hidden.retain(|l| *l != label);
    }
}

/// Show and focus the main window (restoring the secondary windows that were hidden along with
/// it), then tell the frontend so it can select the newest item (8.3).
pub fn show(app: &AppHandle) {
    let restore: Vec<&str> = match HIDDEN_WITH_MAIN.lock() {
        Ok(mut hidden) => hidden.drain(..).collect(),
        Err(_) => Vec::new(),
    };
    for label in restore {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.show();
        }
    }
    if let Some(window) = main_window(app) {
        let _ = window.show();
        let _ = window.set_focus();
        TauriSink(app.clone()).window_shown();
    }
}

/// Hide the main window and any visible secondary window (they come back with the next `show`).
/// The window geometry is saved here as well as at exit (8.6), since the process may be ended
/// without a graceful exit (logout, kill).
pub fn hide(app: &AppHandle) {
    for label in SECONDARY {
        if let Some(window) = app.get_webview_window(label)
            && window.is_visible().unwrap_or(false)
        {
            let _ = window.hide();
            if let Ok(mut hidden) = HIDDEN_WITH_MAIN.lock() {
                hidden.push(label);
            }
        }
    }
    if let Some(window) = main_window(app) {
        save_geometry(app);
        let _ = window.hide();
    }
}

/// Persist the main window's position and size now.
pub fn save_geometry(app: &AppHandle) {
    use tauri_plugin_window_state::{AppHandleExt, StateFlags};
    if let Err(e) = app.save_window_state(StateFlags::SIZE | StateFlags::POSITION) {
        log::warn!("window: could not save window state: {e}");
    }
}

/// Defensive re-assertion of the main window's floating level (8.1) after the activation
/// policy changes or another window of the app is created.
pub fn reassert_always_on_top(app: &AppHandle) {
    if let Some(window) = main_window(app) {
        if let Err(e) = window.set_always_on_top(true) {
            log::warn!("window: set_always_on_top failed: {e}");
        }
        log::debug!("window: always_on_top = {:?}", window.is_always_on_top());
    }
}

/// Hotkey / tray behaviour: visible → hide, hidden → show (8.2).
pub fn toggle(app: &AppHandle) {
    match main_window(app).and_then(|w| w.is_visible().ok()) {
        Some(true) => hide(app),
        _ => show(app),
    }
}

/// Closing the main window hides it instead of ending the app (8.5).
pub fn install_close_to_hide(app: &AppHandle) {
    if let Some(window) = main_window(app) {
        let handle = app.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                hide(&handle);
            }
        });
    }
}

/// Open the settings window from the tray menu (9.1).
pub fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    open_secondary(
        app,
        SETTINGS,
        "clipbuf - Settings",
        SETTINGS_SIZE,
        (400.0, 400.0),
    )
}

/// Close the settings window after a successful save (9.7). Re-opening builds a fresh one,
/// which reads the stored settings again.
pub fn close_settings(app: &AppHandle) -> tauri::Result<()> {
    close_secondary(app, SETTINGS)
}

/// Open the about window from the tray menu (8.8).
pub fn open_about(app: &AppHandle) -> tauri::Result<()> {
    open_secondary(app, ABOUT, "clipbuf - About", ABOUT_SIZE, (320.0, 280.0))
}

pub fn close_about(app: &AppHandle) -> tauri::Result<()> {
    close_secondary(app, ABOUT)
}

/// Open (creating on first use) a secondary window beside the main one. It is an ordinary,
/// not-always-on-top window that loads the same frontend; the frontend picks the view by label.
fn open_secondary(
    app: &AppHandle,
    label: &str,
    title: &str,
    size: (f64, f64),
    min_size: (f64, f64),
) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(label) {
        forget_hidden(label);
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .title(title)
        .inner_size(size.0, size.1)
        .min_inner_size(min_size.0, min_size.1)
        .always_on_top(false);
    if let Some((x, y)) = main_window(app).and_then(|main| beside_main(&main, size)) {
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }
    let window = builder.build()?;
    window.set_focus()?;
    reassert_always_on_top(app);
    Ok(())
}

fn close_secondary(app: &AppHandle, label: &str) -> tauri::Result<()> {
    forget_hidden(label);
    match app.get_webview_window(label) {
        Some(window) => window.close(),
        None => Ok(()),
    }
}

const SETTINGS_SIZE: (f64, f64) = (480.0, 520.0);
const ABOUT_SIZE: (f64, f64) = (380.0, 320.0);
const GAP: f64 = 16.0;

/// Where to put a secondary window so it is not hidden behind the always-on-top main
/// window: to its right, else to its left, else below it — clamped to the main window's
/// monitor. Logical coordinates.
fn beside_main(main: &WebviewWindow, size: (f64, f64)) -> Option<(f64, f64)> {
    let scale = main.scale_factor().ok()?;
    let pos = main.outer_position().ok()?.to_logical::<f64>(scale);
    let outer = main.outer_size().ok()?.to_logical::<f64>(scale);
    let monitor = main.current_monitor().ok()??;
    let m_pos = monitor.position().to_logical::<f64>(scale);
    let m_size = monitor.size().to_logical::<f64>(scale);
    Some(place_beside(
        (pos.x, pos.y, outer.width, outer.height),
        (m_pos.x, m_pos.y, m_size.width, m_size.height),
        size,
        GAP,
    ))
}

/// Pure placement rule (unit-tested): `main` and `monitor` are `(x, y, w, h)`.
pub fn place_beside(
    main: (f64, f64, f64, f64),
    monitor: (f64, f64, f64, f64),
    size: (f64, f64),
    gap: f64,
) -> (f64, f64) {
    let (mx, my, mw, mh) = main;
    let (sx, sy, sw, sh) = monitor;
    let (w, h) = size;
    let clamp_y = |y: f64| y.clamp(sy, (sy + sh - h).max(sy));
    let clamp_x = |x: f64| x.clamp(sx, (sx + sw - w).max(sx));
    if mx + mw + gap + w <= sx + sw {
        (mx + mw + gap, clamp_y(my))
    } else if mx - gap - w >= sx {
        (mx - gap - w, clamp_y(my))
    } else {
        (clamp_x(mx), clamp_y(my + mh + gap))
    }
}

/// Keep clipbuf out of the Dock; it lives in the menu bar (8.4).
#[cfg(target_os = "macos")]
pub fn apply_activation_policy(app: &AppHandle) {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
}

#[cfg(not(target_os = "macos"))]
pub fn apply_activation_policy(_app: &AppHandle) {}
