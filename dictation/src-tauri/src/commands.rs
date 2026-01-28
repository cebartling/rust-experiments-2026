use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;

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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), DictationError> {
    let mut current = state.settings.lock().unwrap();
    *current = settings.clone();

    if let Ok(store) = app.store("settings.json") {
        let value = serde_json::to_value(&settings)
            .map_err(|e| DictationError::Config(format!("serialize settings: {e}")))?;
        store.set("settings", value);
    }

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

#[tauri::command]
pub fn start_dictation(state: State<'_, AppState>) -> Result<(), DictationError> {
    state.ensure_recorder()?;

    let mut manager = state.dictation_manager.lock().unwrap();
    manager.on_hotkey_pressed()?;
    drop(manager);

    let mut guard = state.recorder.lock().unwrap();
    if let Some(recorder) = guard.as_mut() {
        recorder.start()?;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_dictation(app: tauri::AppHandle) -> Result<String, DictationError> {
    // Phase 1: Synchronous — stop recording and collect audio
    let (audio, sample_rate, channels, settings) = {
        let state = app.state::<AppState>();

        let mut manager = state.dictation_manager.lock().unwrap();
        manager.on_hotkey_released()?;
        drop(manager);

        let (audio, sr, ch) = {
            let mut guard = state.recorder.lock().unwrap();
            let rec = guard
                .as_mut()
                .ok_or_else(|| DictationError::Audio("no recorder available".into()))?;
            let sr = rec.sample_rate();
            let ch = rec.channels();
            let audio = rec.stop()?;
            (audio, sr, ch)
        };

        let settings = state.settings.lock().unwrap().clone();
        (audio, sr, ch, settings)
    };

    // Phase 2: Async — resample, transcribe, inject
    let resampled = crate::audio::resample::resample_to_16khz_mono(&audio, sample_rate, channels)?;
    if resampled.is_empty() {
        let state = app.state::<AppState>();
        state.dictation_manager.lock().unwrap().on_error();
        return Err(DictationError::Audio("no audio captured".into()));
    }

    let engine = match crate::stt::create_engine(&settings) {
        Ok(e) => e,
        Err(e) => {
            let state = app.state::<AppState>();
            state.dictation_manager.lock().unwrap().on_error();
            return Err(e);
        }
    };

    let result = match engine.transcribe(&resampled).await {
        Ok(r) => r,
        Err(e) => {
            let state = app.state::<AppState>();
            state.dictation_manager.lock().unwrap().on_error();
            return Err(e);
        }
    };

    // Phase 3: Inject text and complete
    {
        let state = app.state::<AppState>();
        if settings.auto_inject
            && !result.text.is_empty()
            && let Err(e) = state.injector.inject(&result.text)
        {
            state.dictation_manager.lock().unwrap().on_error();
            return Err(e);
        }
        let mut manager = state.dictation_manager.lock().unwrap();
        if let Err(e) = manager.on_transcription_complete() {
            eprintln!("transcription complete transition: {e}");
        }
    }

    Ok(result.text)
}

#[tauri::command]
pub fn reset_dictation(state: State<'_, AppState>) -> Result<(), DictationError> {
    let mut manager = state.dictation_manager.lock().unwrap();
    manager.reset()
}

#[tauri::command]
pub fn get_dictation_state(state: State<'_, AppState>) -> String {
    let manager = state.dictation_manager.lock().unwrap();
    manager.state().to_string()
}
