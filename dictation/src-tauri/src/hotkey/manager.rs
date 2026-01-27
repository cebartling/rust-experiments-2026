use serde::Serialize;

use crate::error::DictationError;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DictationState {
    Idle,
    Recording,
    Transcribing,
    Error,
}

impl std::fmt::Display for DictationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictationState::Idle => write!(f, "idle"),
            DictationState::Recording => write!(f, "recording"),
            DictationState::Transcribing => write!(f, "transcribing"),
            DictationState::Error => write!(f, "error"),
        }
    }
}

pub struct DictationManager {
    state: DictationState,
}

impl Default for DictationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DictationManager {
    pub fn new() -> Self {
        Self {
            state: DictationState::Idle,
        }
    }

    pub fn state(&self) -> &DictationState {
        &self.state
    }

    pub fn on_hotkey_pressed(&mut self) -> Result<(), DictationError> {
        match self.state {
            DictationState::Idle => {
                self.state = DictationState::Recording;
                Ok(())
            }
            _ => Err(DictationError::InvalidStateTransition {
                from: self.state.to_string(),
                to: "recording".into(),
            }),
        }
    }

    pub fn on_hotkey_released(&mut self) -> Result<(), DictationError> {
        match self.state {
            DictationState::Recording => {
                self.state = DictationState::Transcribing;
                Ok(())
            }
            _ => Err(DictationError::InvalidStateTransition {
                from: self.state.to_string(),
                to: "transcribing".into(),
            }),
        }
    }

    pub fn on_transcription_complete(&mut self) -> Result<(), DictationError> {
        match self.state {
            DictationState::Transcribing => {
                self.state = DictationState::Idle;
                Ok(())
            }
            _ => Err(DictationError::InvalidStateTransition {
                from: self.state.to_string(),
                to: "idle".into(),
            }),
        }
    }

    pub fn on_error(&mut self) {
        self.state = DictationState::Error;
    }

    pub fn reset(&mut self) -> Result<(), DictationError> {
        match self.state {
            DictationState::Error => {
                self.state = DictationState::Idle;
                Ok(())
            }
            _ => Err(DictationError::InvalidStateTransition {
                from: self.state.to_string(),
                to: "idle".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_is_idle() {
        let manager = DictationManager::new();
        assert_eq!(*manager.state(), DictationState::Idle);
    }

    // Happy path: Idle -> Recording -> Transcribing -> Idle
    #[test]
    fn full_dictation_cycle() {
        let mut manager = DictationManager::new();

        manager.on_hotkey_pressed().unwrap();
        assert_eq!(*manager.state(), DictationState::Recording);

        manager.on_hotkey_released().unwrap();
        assert_eq!(*manager.state(), DictationState::Transcribing);

        manager.on_transcription_complete().unwrap();
        assert_eq!(*manager.state(), DictationState::Idle);
    }

    // Error recovery: Idle -> Recording -> Transcribing -> Error -> Idle
    #[test]
    fn error_recovery_cycle() {
        let mut manager = DictationManager::new();
        manager.on_hotkey_pressed().unwrap();
        manager.on_hotkey_released().unwrap();

        manager.on_error();
        assert_eq!(*manager.state(), DictationState::Error);

        manager.reset().unwrap();
        assert_eq!(*manager.state(), DictationState::Idle);
    }

    // Invalid transitions
    #[test]
    fn cannot_press_hotkey_while_recording() {
        let mut manager = DictationManager::new();
        manager.on_hotkey_pressed().unwrap();
        let result = manager.on_hotkey_pressed();
        assert!(result.is_err());
    }

    #[test]
    fn cannot_release_hotkey_while_idle() {
        let mut manager = DictationManager::new();
        let result = manager.on_hotkey_released();
        assert!(result.is_err());
    }

    #[test]
    fn cannot_complete_transcription_while_idle() {
        let mut manager = DictationManager::new();
        let result = manager.on_transcription_complete();
        assert!(result.is_err());
    }

    #[test]
    fn cannot_reset_from_idle() {
        let mut manager = DictationManager::new();
        let result = manager.reset();
        assert!(result.is_err());
    }

    #[test]
    fn cannot_press_hotkey_while_transcribing() {
        let mut manager = DictationManager::new();
        manager.on_hotkey_pressed().unwrap();
        manager.on_hotkey_released().unwrap();
        let result = manager.on_hotkey_pressed();
        assert!(result.is_err());
    }

    #[test]
    fn error_from_any_state_via_on_error() {
        let mut manager = DictationManager::new();
        manager.on_error();
        assert_eq!(*manager.state(), DictationState::Error);
    }

    #[test]
    fn state_display_format() {
        assert_eq!(DictationState::Idle.to_string(), "idle");
        assert_eq!(DictationState::Recording.to_string(), "recording");
        assert_eq!(DictationState::Transcribing.to_string(), "transcribing");
        assert_eq!(DictationState::Error.to_string(), "error");
    }

    #[test]
    fn state_serializes_lowercase() {
        let json = serde_json::to_string(&DictationState::Recording).unwrap();
        assert_eq!(json, "\"recording\"");
    }

    // Multiple full cycles
    #[test]
    fn can_run_multiple_cycles() {
        let mut manager = DictationManager::new();
        for _ in 0..3 {
            manager.on_hotkey_pressed().unwrap();
            manager.on_hotkey_released().unwrap();
            manager.on_transcription_complete().unwrap();
        }
        assert_eq!(*manager.state(), DictationState::Idle);
    }
}
