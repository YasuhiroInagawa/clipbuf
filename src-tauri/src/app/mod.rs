//! Tauri runtime wiring: state, capture service, commands, events, tray, window, hotkey.

pub mod events;
pub mod platform;

#[cfg(test)]
mod platform_test;
