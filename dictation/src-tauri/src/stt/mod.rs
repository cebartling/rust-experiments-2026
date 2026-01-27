pub mod cloud;
pub mod local;

use async_trait::async_trait;
use serde::Serialize;

use crate::config::{AppSettings, SttBackendChoice};
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

/// Creates an STT engine based on the current application settings.
pub fn create_engine(settings: &AppSettings) -> Result<Box<dyn SttEngine>, DictationError> {
    match settings.stt_backend {
        SttBackendChoice::Local => {
            let engine = if settings.language.is_empty() {
                local::WhisperLocal::new(settings.local_model_path.clone())?
            } else {
                local::WhisperLocal::with_language(
                    settings.local_model_path.clone(),
                    settings.language.clone(),
                )?
            };
            Ok(Box::new(engine))
        }
        SttBackendChoice::Cloud => {
            let engine = if settings.language.is_empty() {
                cloud::CloudStt::new(settings.cloud_api_key.clone(), settings.cloud_model.clone())?
            } else {
                cloud::CloudStt::with_language(
                    settings.cloud_api_key.clone(),
                    settings.cloud_model.clone(),
                    settings.language.clone(),
                )?
            };
            Ok(Box::new(engine))
        }
    }
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

    #[test]
    fn create_engine_local_with_valid_path() {
        let mut settings = AppSettings::default();
        settings.stt_backend = SttBackendChoice::Local;
        settings.local_model_path = "/path/to/model.bin".into();
        let engine = create_engine(&settings).unwrap();
        assert_eq!(engine.kind(), SttBackendKind::Local);
    }

    #[test]
    fn create_engine_local_rejects_empty_path() {
        let mut settings = AppSettings::default();
        settings.stt_backend = SttBackendChoice::Local;
        settings.local_model_path = String::new();
        let result = create_engine(&settings);
        assert!(result.is_err());
    }

    #[test]
    fn create_engine_cloud_with_valid_key() {
        let mut settings = AppSettings::default();
        settings.stt_backend = SttBackendChoice::Cloud;
        settings.cloud_api_key = "sk-test-key".into();
        settings.cloud_model = "whisper-1".into();
        let engine = create_engine(&settings).unwrap();
        assert_eq!(engine.kind(), SttBackendKind::Cloud);
    }

    #[test]
    fn create_engine_cloud_rejects_empty_key() {
        let mut settings = AppSettings::default();
        settings.stt_backend = SttBackendChoice::Cloud;
        settings.cloud_api_key = String::new();
        let result = create_engine(&settings);
        assert!(result.is_err());
    }
}
