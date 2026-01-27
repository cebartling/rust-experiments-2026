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
pub async fn test_stt_backend(state: State<'_, AppState>) -> Result<String, DictationError> {
    let settings = state.settings.lock().unwrap().clone();
    let engine = crate::stt::create_engine(&settings)?;
    engine.health_check().await?;
    Ok(format!("{:?} backend is healthy", engine.kind()))
}

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<String>, DictationError> {
    crate::audio::capture::list_input_devices()
}
