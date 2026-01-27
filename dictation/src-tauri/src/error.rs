use thiserror::Error;

#[derive(Debug, Error)]
pub enum DictationError {
    #[error("Audio error: {0}")]
    Audio(String),

    #[error("STT error: {0}")]
    Stt(String),

    #[error("Injection error: {0}")]
    Injection(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: String, to: String },
}

impl serde::Serialize for DictationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_error_displays_message() {
        let err = DictationError::Audio("mic not found".into());
        assert_eq!(err.to_string(), "Audio error: mic not found");
    }

    #[test]
    fn stt_error_displays_message() {
        let err = DictationError::Stt("model load failed".into());
        assert_eq!(err.to_string(), "STT error: model load failed");
    }

    #[test]
    fn injection_error_displays_message() {
        let err = DictationError::Injection("keyboard unavailable".into());
        assert_eq!(err.to_string(), "Injection error: keyboard unavailable");
    }

    #[test]
    fn invalid_state_transition_displays_details() {
        let err = DictationError::InvalidStateTransition {
            from: "Idle".into(),
            to: "Transcribing".into(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid state transition: \"Idle\" -> \"Transcribing\""
        );
    }

    #[test]
    fn error_serializes_as_string() {
        let err = DictationError::Audio("test".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"Audio error: test\"");
    }
}
