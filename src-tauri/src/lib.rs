//! clipbuf core: clipboard monitoring, analysis, transformation and the Tauri runtime.
//!
//! Dependency direction (see design.md): `model` → `analysis` / `transform` / `buffer`
//! → `clipboard` / `settings` → `app`. Modules never import from a layer to their right.

pub mod analysis;
pub mod app;
pub mod buffer;
pub mod clipboard;
pub mod model;
pub mod settings;
pub mod transform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
