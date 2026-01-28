use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{ShortcutEvent, ShortcutState};

use crate::audio::resample::resample_to_16khz_mono;
use crate::events::{
    DICTATION_ERROR, DICTATION_STATE_CHANGED, DictationStatePayload, ErrorPayload,
    TRANSCRIPTION_COMPLETE, TranscriptionPayload,
};
use crate::state::AppState;

/// Top-level handler for global shortcut events.
/// Dispatches to press/release handlers based on the event state.
pub fn handle_shortcut_event<R: Runtime>(
    app: &AppHandle<R>,
    _shortcut: &tauri_plugin_global_shortcut::Shortcut,
    event: ShortcutEvent,
) {
    match event.state {
        ShortcutState::Pressed => handle_press(app),
        ShortcutState::Released => handle_release(app),
    }
}

/// Hotkey pressed: initialize recorder if needed, transition to Recording,
/// start audio capture, and emit state event.
fn handle_press<R: Runtime>(app: &AppHandle<R>) {
    let app_state = app.state::<AppState>();

    // Ensure recorder is available (lazy init)
    if let Err(e) = app_state.ensure_recorder() {
        let mut manager = app_state.dictation_manager.lock().unwrap();
        manager.on_error();
        drop(manager);
        emit_error(app, &e.to_string());
        emit_state(app, "error", None);
        return;
    }

    // Transition state machine: Idle → Recording
    {
        let mut manager = app_state.dictation_manager.lock().unwrap();
        if let Err(e) = manager.on_hotkey_pressed() {
            eprintln!("hotkey press: {e}");
            return;
        }
    }

    // Start recording
    {
        let mut recorder_guard = app_state.recorder.lock().unwrap();
        if let Some(recorder) = recorder_guard.as_mut()
            && let Err(e) = recorder.start()
        {
            let mut manager = app_state.dictation_manager.lock().unwrap();
            manager.on_error();
            drop(manager);
            emit_error(app, &e.to_string());
            emit_state(app, "error", None);
            return;
        }
    }

    emit_state(app, "recording", None);
}

/// Hotkey released: transition to Transcribing, stop capture,
/// then spawn async task for resample → transcribe → inject.
fn handle_release<R: Runtime>(app: &AppHandle<R>) {
    let app_state = app.state::<AppState>();

    // Transition state machine: Recording → Transcribing
    {
        let mut manager = app_state.dictation_manager.lock().unwrap();
        if let Err(e) = manager.on_hotkey_released() {
            eprintln!("hotkey release: {e}");
            return;
        }
    }

    emit_state(app, "transcribing", None);

    // Stop recording and collect audio + metadata
    let (audio, sample_rate, channels) = {
        let mut recorder_guard = app_state.recorder.lock().unwrap();
        let recorder = match recorder_guard.as_mut() {
            Some(r) => r,
            None => {
                pipeline_error(app, "no recorder available");
                return;
            }
        };
        let sample_rate = recorder.sample_rate();
        let channels = recorder.channels();
        match recorder.stop() {
            Ok(audio) => (audio, sample_rate, channels),
            Err(e) => {
                pipeline_error(app, &e.to_string());
                return;
            }
        }
    };

    // Snapshot settings for the async task
    let settings = app_state.settings.lock().unwrap().clone();

    // Spawn async task: resample → transcribe → inject
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        // Resample to 16kHz mono
        let resampled = match resample_to_16khz_mono(&audio, sample_rate, channels) {
            Ok(r) => r,
            Err(e) => {
                pipeline_error(&app_handle, &e.to_string());
                return;
            }
        };

        if resampled.is_empty() {
            pipeline_error(&app_handle, "no audio captured");
            return;
        }

        // Create STT engine from settings
        let engine = match crate::stt::create_engine(&settings) {
            Ok(e) => e,
            Err(e) => {
                pipeline_error(&app_handle, &e.to_string());
                return;
            }
        };

        // Transcribe
        let result = match engine.transcribe(&resampled).await {
            Ok(r) => r,
            Err(e) => {
                pipeline_error(&app_handle, &e.to_string());
                return;
            }
        };

        // Inject text into focused application (if enabled and non-empty)
        if settings.auto_inject && !result.text.is_empty() {
            let state = app_handle.state::<AppState>();
            if let Err(e) = state.injector.inject(&result.text) {
                pipeline_error(&app_handle, &e.to_string());
                return;
            }
        }

        // Transition state machine: Transcribing → Idle
        {
            let state = app_handle.state::<AppState>();
            let mut manager = state.dictation_manager.lock().unwrap();
            if let Err(e) = manager.on_transcription_complete() {
                eprintln!("transcription complete: {e}");
            }
        }

        // Emit completion events
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .to_string();

        let _ = app_handle.emit(
            TRANSCRIPTION_COMPLETE,
            TranscriptionPayload {
                text: result.text,
                timestamp,
            },
        );
        emit_state(&app_handle, "idle", None);
    });
}

/// Handles a pipeline error: transitions state machine to Error,
/// emits error and state events.
fn pipeline_error<R: Runtime>(app: &AppHandle<R>, message: &str) {
    let state = app.state::<AppState>();
    {
        let mut manager = state.dictation_manager.lock().unwrap();
        manager.on_error();
    }
    emit_error(app, message);
    emit_state(app, "error", Some(message));
}

fn emit_state<R: Runtime>(app: &AppHandle<R>, state: &str, message: Option<&str>) {
    let _ = app.emit(
        DICTATION_STATE_CHANGED,
        DictationStatePayload {
            state: state.into(),
            message: message.map(String::from),
        },
    );
}

fn emit_error<R: Runtime>(app: &AppHandle<R>, message: &str) {
    let _ = app.emit(
        DICTATION_ERROR,
        ErrorPayload {
            message: message.into(),
        },
    );
}

#[cfg(test)]
mod tests {
    use crate::hotkey::DictationState;
    use crate::state::AppState;

    #[test]
    fn pipeline_error_sets_error_state() {
        // Verify the state machine transition logic that pipeline_error relies on
        let state = AppState::new();
        {
            let mut manager = state.dictation_manager.lock().unwrap();
            manager.on_error();
        }
        let manager = state.dictation_manager.lock().unwrap();
        assert_eq!(*manager.state(), DictationState::Error);
    }

    #[test]
    fn state_machine_full_cycle_without_tauri() {
        // Verify the state transitions the pipeline would make
        let state = AppState::new();

        // Press: Idle → Recording
        {
            let mut manager = state.dictation_manager.lock().unwrap();
            manager.on_hotkey_pressed().unwrap();
            assert_eq!(*manager.state(), DictationState::Recording);
        }

        // Release: Recording → Transcribing
        {
            let mut manager = state.dictation_manager.lock().unwrap();
            manager.on_hotkey_released().unwrap();
            assert_eq!(*manager.state(), DictationState::Transcribing);
        }

        // Complete: Transcribing → Idle
        {
            let mut manager = state.dictation_manager.lock().unwrap();
            manager.on_transcription_complete().unwrap();
            assert_eq!(*manager.state(), DictationState::Idle);
        }
    }

    #[test]
    fn state_machine_error_during_transcribing() {
        let state = AppState::new();

        // Press → Release → Error → Reset
        {
            let mut manager = state.dictation_manager.lock().unwrap();
            manager.on_hotkey_pressed().unwrap();
            manager.on_hotkey_released().unwrap();
            manager.on_error();
            assert_eq!(*manager.state(), DictationState::Error);
            manager.reset().unwrap();
            assert_eq!(*manager.state(), DictationState::Idle);
        }
    }
}
