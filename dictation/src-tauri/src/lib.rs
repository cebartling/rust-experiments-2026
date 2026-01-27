pub mod audio;
pub mod commands;
pub mod config;
pub mod error;
pub mod events;
pub mod hotkey;
pub mod injection;
pub mod state;
pub mod stt;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::test_stt_backend,
            commands::get_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
