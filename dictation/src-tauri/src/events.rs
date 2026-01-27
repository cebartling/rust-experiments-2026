use serde::Serialize;

pub const DICTATION_STATE_CHANGED: &str = "dictation-state-changed";
pub const TRANSCRIPTION_COMPLETE: &str = "transcription-complete";
pub const DICTATION_ERROR: &str = "dictation-error";

#[derive(Debug, Clone, Serialize)]
pub struct DictationStatePayload {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionPayload {
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_names_are_kebab_case() {
        assert_eq!(DICTATION_STATE_CHANGED, "dictation-state-changed");
        assert_eq!(TRANSCRIPTION_COMPLETE, "transcription-complete");
        assert_eq!(DICTATION_ERROR, "dictation-error");
    }

    #[test]
    fn state_payload_serializes_correctly() {
        let payload = DictationStatePayload {
            state: "recording".into(),
            message: None,
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["state"], "recording");
        assert!(json.get("message").is_none());
    }

    #[test]
    fn state_payload_includes_message_when_present() {
        let payload = DictationStatePayload {
            state: "error".into(),
            message: Some("mic failed".into()),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["state"], "error");
        assert_eq!(json["message"], "mic failed");
    }

    #[test]
    fn transcription_payload_serializes_correctly() {
        let payload = TranscriptionPayload {
            text: "hello world".into(),
            timestamp: "2026-01-27T12:00:00Z".into(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["text"], "hello world");
        assert_eq!(json["timestamp"], "2026-01-27T12:00:00Z");
    }

    #[test]
    fn error_payload_serializes_correctly() {
        let payload = ErrorPayload {
            message: "something went wrong".into(),
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["message"], "something went wrong");
    }
}
