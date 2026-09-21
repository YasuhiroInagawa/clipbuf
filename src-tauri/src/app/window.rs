//! Main and settings windows (8.1–8.5, 9.1).
//!
//! The main window is configured in `tauri.conf.json` (always on top, skip taskbar, all
//! workspaces). Closing hides it; the tray and the hotkey toggle it. The settings window is a
//! second webview created on first use.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::events::{EventSink, TauriSink};

pub const MAIN: &str = "main";
pub const SETTINGS: &str = "settings";

/// Set when hiding the main window also hid an open settings window, so that showing the
/// main window brings the settings window back with it. Hiding clipbuf means "get out of the
/// way"; the settings window only goes away for good when the user closes it.
static SETTINGS_HIDDEN_WITH_MAIN: AtomicBool = AtomicBool::new(false);

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(MAIN)
}

fn settings_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(SETTINGS)
}

/// Show and focus the main window (restoring a settings window that was hidden along with it),
/// then tell the frontend so it can select the newest item (8.3).
pub fn show(app: &AppHandle) {
    if SETTINGS_HIDDEN_WITH_MAIN.swap(false, Ordering::SeqCst)
        && let Some(settings) = settings_window(app)
    {
        let _ = settings.show();
    }
    if let Some(window) = main_window(app) {
        let _ = window.show();
        let _ = window.set_focus();
        TauriSink(app.clone()).window_shown();
    }
}

/// Hide the main window and, if the settings window is open, hide it too (it comes back with
/// the next `show`).
pub fn hide(app: &AppHandle) {
    if let Some(settings) = settings_window(app)
        && settings.is_visible().unwrap_or(false)
    {
        let _ = settings.hide();
        SETTINGS_HIDDEN_WITH_MAIN.store(true, Ordering::SeqCst);
    }
    if let Some(window) = main_window(app) {
        let _ = window.hide();
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

/// Open (creating on first use) the settings window. It is an ordinary, not-always-on-top
/// window that loads the same frontend; the frontend picks the view by window label.
pub fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = settings_window(app) {
        SETTINGS_HIDDEN_WITH_MAIN.store(false, Ordering::SeqCst);
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    let mut builder = WebviewWindowBuilder::new(app, SETTINGS, WebviewUrl::default())
        .title("clipbuf - Settings")
        .inner_size(SETTINGS_SIZE.0, SETTINGS_SIZE.1)
        .min_inner_size(400.0, 400.0)
        .always_on_top(false);
    if let Some((x, y)) = main_window(app).and_then(|main| settings_position(&main)) {
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }
    let window = builder.build()?;
    window.set_focus()?;
    reassert_always_on_top(app);
    Ok(())
}

const SETTINGS_SIZE: (f64, f64) = (480.0, 520.0);
const GAP: f64 = 16.0;

/// Where to put the settings window so it is not hidden behind the always-on-top main
/// window: to its right, else to its left, else below it — clamped to the main window's
/// monitor. Logical coordinates.
fn settings_position(main: &WebviewWindow) -> Option<(f64, f64)> {
    let scale = main.scale_factor().ok()?;
    let pos = main.outer_position().ok()?.to_logical::<f64>(scale);
    let size = main.outer_size().ok()?.to_logical::<f64>(scale);
    let monitor = main.current_monitor().ok()??;
    let m_pos = monitor.position().to_logical::<f64>(scale);
    let m_size = monitor.size().to_logical::<f64>(scale);
    Some(place_beside(
        (pos.x, pos.y, size.width, size.height),
        (m_pos.x, m_pos.y, m_size.width, m_size.height),
        SETTINGS_SIZE,
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
