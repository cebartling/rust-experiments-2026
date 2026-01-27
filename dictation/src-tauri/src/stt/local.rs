use async_trait::async_trait;

use super::{SttBackendKind, SttEngine, TranscriptionResult};
use crate::error::DictationError;

pub struct WhisperLocal {
    model_path: String,
}

impl WhisperLocal {
    pub fn new(model_path: String) -> Result<Self, DictationError> {
        if model_path.is_empty() {
            return Err(DictationError::Stt("model path is empty".into()));
        }
        Ok(Self { model_path })
    }

    pub fn model_path(&self) -> &str {
        &self.model_path
    }
}

#[async_trait]
impl SttEngine for WhisperLocal {
    async fn transcribe(
        &self,
        _audio_16khz_mono: &[f32],
    ) -> Result<TranscriptionResult, DictationError> {
        // TODO: Phase 3 - implement whisper-rs transcription
        Err(DictationError::Stt(
            "local transcription not yet implemented".into(),
        ))
    }

    fn kind(&self) -> SttBackendKind {
        SttBackendKind::Local
    }

    async fn health_check(&self) -> Result<(), DictationError> {
        // TODO: Phase 3 - verify model file exists and loads
        Err(DictationError::Stt(
            "local health check not yet implemented".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty_model_path() {
        let result = WhisperLocal::new(String::new());
        assert!(result.is_err());
    }

    #[test]
    fn new_accepts_valid_model_path() {
        let engine = WhisperLocal::new("/path/to/model.bin".into()).unwrap();
        assert_eq!(engine.model_path(), "/path/to/model.bin");
    }

    #[test]
    fn kind_returns_local() {
        let engine = WhisperLocal::new("/path/to/model.bin".into()).unwrap();
        assert_eq!(engine.kind(), SttBackendKind::Local);
    }

    #[tokio::test]
    async fn transcribe_returns_not_implemented() {
        let engine = WhisperLocal::new("/path/to/model.bin".into()).unwrap();
        let result = engine.transcribe(&[]).await;
        assert!(result.is_err());
    }
}
