use async_trait::async_trait;

use super::{SttBackendKind, SttEngine, TranscriptionResult};
use crate::error::DictationError;

pub struct CloudStt {
    api_key: String,
    model: String,
}

impl CloudStt {
    pub fn new(api_key: String, model: String) -> Result<Self, DictationError> {
        if api_key.is_empty() {
            return Err(DictationError::Stt("API key is empty".into()));
        }
        Ok(Self { api_key, model })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn has_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }
}

#[async_trait]
impl SttEngine for CloudStt {
    async fn transcribe(
        &self,
        _audio_16khz_mono: &[f32],
    ) -> Result<TranscriptionResult, DictationError> {
        // TODO: Phase 3 - implement OpenAI Whisper API call via reqwest
        Err(DictationError::Stt(
            "cloud transcription not yet implemented".into(),
        ))
    }

    fn kind(&self) -> SttBackendKind {
        SttBackendKind::Cloud
    }

    async fn health_check(&self) -> Result<(), DictationError> {
        // TODO: Phase 3 - verify API key works with a test request
        Err(DictationError::Stt(
            "cloud health check not yet implemented".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty_api_key() {
        let result = CloudStt::new(String::new(), "whisper-1".into());
        assert!(result.is_err());
    }

    #[test]
    fn new_accepts_valid_credentials() {
        let engine = CloudStt::new("sk-test-key".into(), "whisper-1".into()).unwrap();
        assert_eq!(engine.model(), "whisper-1");
        assert!(engine.has_api_key());
    }

    #[test]
    fn kind_returns_cloud() {
        let engine = CloudStt::new("sk-test-key".into(), "whisper-1".into()).unwrap();
        assert_eq!(engine.kind(), SttBackendKind::Cloud);
    }

    #[tokio::test]
    async fn transcribe_returns_not_implemented() {
        let engine = CloudStt::new("sk-test-key".into(), "whisper-1".into()).unwrap();
        let result = engine.transcribe(&[]).await;
        assert!(result.is_err());
    }
}
