pub mod cloud;
pub mod local;

use async_trait::async_trait;
use serde::Serialize;

use crate::error::DictationError;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SttBackendKind {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
}

#[async_trait]
pub trait SttEngine: Send + Sync {
    async fn transcribe(
        &self,
        audio_16khz_mono: &[f32],
    ) -> Result<TranscriptionResult, DictationError>;
    fn kind(&self) -> SttBackendKind;
    async fn health_check(&self) -> Result<(), DictationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_kind_serializes_lowercase() {
        let local = serde_json::to_string(&SttBackendKind::Local).unwrap();
        assert_eq!(local, "\"local\"");
        let cloud = serde_json::to_string(&SttBackendKind::Cloud).unwrap();
        assert_eq!(cloud, "\"cloud\"");
    }

    #[test]
    fn transcription_result_serializes_correctly() {
        let result = TranscriptionResult {
            text: "hello".into(),
            language: Some("en".into()),
            duration_ms: 1500,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["text"], "hello");
        assert_eq!(json["language"], "en");
        assert_eq!(json["duration_ms"], 1500);
    }

    #[test]
    fn transcription_result_handles_no_language() {
        let result = TranscriptionResult {
            text: "test".into(),
            language: None,
            duration_ms: 500,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["text"], "test");
        assert!(json["language"].is_null());
    }
}
