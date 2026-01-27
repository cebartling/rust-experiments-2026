use std::sync::Mutex;

use crate::config::AppSettings;
use crate::hotkey::DictationManager;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub dictation_manager: Mutex<DictationManager>,
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
        }
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
}
