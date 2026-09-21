//! Tauri runtime wiring: state, capture service, commands, events, tray, window, hotkey.

pub mod capture;
pub mod events;
pub mod platform;
pub mod state;

#[cfg(test)]
mod capture_test;
#[cfg(test)]
mod platform_test;
