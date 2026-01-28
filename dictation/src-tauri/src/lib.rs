pub mod audio;
pub mod commands;
pub mod config;
pub mod error;
pub mod events;
pub mod hotkey;
pub mod injection;
pub mod pipeline;
pub mod state;
pub mod stt;

use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_store::StoreExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            // Load persisted settings from store (if any)
            if let Ok(store) = app.store("settings.json")
                && let Some(value) = store.get("settings")
                && let Ok(settings) = serde_json::from_value::<config::AppSettings>(value.clone())
            {
                let state = app.state::<AppState>();
                *state.settings.lock().unwrap() = settings;
            }

            let hotkey = {
                let state = app.state::<AppState>();
                let settings = state.settings.lock().unwrap();
                settings.hotkey.clone()
            };
            app.global_shortcut()
                .on_shortcut(hotkey.as_str(), |app, shortcut, event| {
                    pipeline::handle_shortcut_event(app, shortcut, event);
                })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::test_stt_backend,
            commands::get_audio_devices,
            commands::start_dictation,
            commands::stop_dictation,
            commands::reset_dictation,
            commands::get_dictation_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
