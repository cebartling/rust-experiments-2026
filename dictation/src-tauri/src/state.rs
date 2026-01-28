use std::sync::Mutex;

use crate::audio::capture::AudioRecorder;
use crate::config::AppSettings;
use crate::hotkey::DictationManager;
use crate::injection::keyboard::TextInjector;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub dictation_manager: Mutex<DictationManager>,
    pub recorder: Mutex<Option<AudioRecorder>>,
    pub injector: TextInjector,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            settings: Mutex::new(AppSettings::default()),
            dictation_manager: Mutex::new(DictationManager::new()),
            recorder: Mutex::new(None),
            injector: TextInjector::new(),
        }
    }

    /// Ensures an AudioRecorder is available, lazily creating one
    /// bound to the default input device.
    pub fn ensure_recorder(&self) -> Result<(), crate::error::DictationError> {
        let mut guard = self.recorder.lock().unwrap();
        if guard.is_none() {
            let recorder = AudioRecorder::with_default_device()?;
            *guard = Some(recorder);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hotkey::DictationState;

    #[test]
    fn new_state_has_default_settings() {
        let state = AppState::new();
        let settings = state.settings.lock().unwrap();
        assert_eq!(settings.hotkey, "Ctrl+Shift+Space");
    }

    #[test]
    fn new_state_has_idle_dictation_manager() {
        let state = AppState::new();
        let manager = state.dictation_manager.lock().unwrap();
        assert_eq!(*manager.state(), DictationState::Idle);
    }

    #[test]
    fn new_state_has_no_recorder() {
        let state = AppState::new();
        let recorder = state.recorder.lock().unwrap();
        assert!(recorder.is_none());
    }

    #[test]
    fn new_state_has_injector() {
        let state = AppState::new();
        // TextInjector always constructs successfully
        assert!(state.injector.inject("").is_ok());
    }
}
