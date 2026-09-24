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
    // Development builds log to stderr (warn+ by default, RUST_LOG overrides). Release builds
    // stay quiet; item content is never logged in either (10.5).
    if cfg!(debug_assertions) {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .try_init();
    }
    tauri::Builder::default()
        // Must be the first plugin: a second launch hands its args to the running instance,
        // which brings the main window forward.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app::window::show(app);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![app::HIDDEN_FLAG]),
        ))
        // Position and size of the main window survive restarts (8.6). The settings window is
        // placed next to the main window instead, and visibility is managed by `app::window`.
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION,
                )
                .with_denylist(&[app::window::SETTINGS])
                .build(),
        )
        .setup(|app| {
            app::bootstrap(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app::commands::list_items,
            app::commands::transfer_item,
            app::commands::preview_transfer,
            app::commands::remove_item,
            app::commands::clear_items,
            app::commands::get_settings,
            app::commands::set_settings,
            app::commands::get_platform_info,
            app::commands::hide_window,
            app::commands::open_settings,
            app::commands::close_settings,
            app::commands::suspend_hotkey,
            app::commands::resume_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
