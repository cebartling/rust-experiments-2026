use tauri::State;

use crate::config::AppSettings;
use crate::error::DictationError;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, DictationError> {
    let settings = state.settings.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), DictationError> {
    let mut current = state.settings.lock().unwrap();
    *current = settings;
    Ok(())
}

#[tauri::command]
pub fn test_stt_backend() -> Result<String, DictationError> {
    // TODO: Phase 3 - run a health check on the configured STT backend
    Err(DictationError::Stt(
        "STT backend test not yet implemented".into(),
    ))
}

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<String>, DictationError> {
    crate::audio::capture::list_input_devices()
}
