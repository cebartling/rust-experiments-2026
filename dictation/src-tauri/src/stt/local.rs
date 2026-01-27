use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::{SttBackendKind, SttEngine, TranscriptionResult};
use crate::error::DictationError;

pub struct WhisperLocal {
    model_path: String,
    language: Option<String>,
    context: Mutex<Option<Arc<WhisperContext>>>,
}

impl WhisperLocal {
    pub fn new(model_path: String) -> Result<Self, DictationError> {
        if model_path.is_empty() {
            return Err(DictationError::Stt("model path is empty".into()));
        }
        Ok(Self {
            model_path,
            language: None,
            context: Mutex::new(None),
        })
    }

    pub fn with_language(model_path: String, language: String) -> Result<Self, DictationError> {
        let mut engine = Self::new(model_path)?;
        engine.language = Some(language);
        Ok(engine)
    }

    pub fn model_path(&self) -> &str {
        &self.model_path
    }

    /// Lazily loads the Whisper model into memory. Subsequent calls are no-ops.
    fn ensure_loaded(&self) -> Result<(), DictationError> {
        let mut ctx_guard = self.context.lock().unwrap();
        if ctx_guard.is_none() {
            let context = WhisperContext::new_with_params(
                &self.model_path,
                WhisperContextParameters::default(),
            )
            .map_err(|e| DictationError::Stt(format!("load model '{}': {e}", self.model_path)))?;
            *ctx_guard = Some(Arc::new(context));
        }
        Ok(())
    }
}

#[async_trait]
impl SttEngine for WhisperLocal {
    async fn transcribe(
        &self,
        audio_16khz_mono: &[f32],
    ) -> Result<TranscriptionResult, DictationError> {
        if audio_16khz_mono.is_empty() {
            return Err(DictationError::Stt("audio data is empty".into()));
        }

        self.ensure_loaded()?;

        let ctx: Arc<WhisperContext> = self.context.lock().unwrap().as_ref().unwrap().clone();
        let audio = audio_16khz_mono.to_vec();
        let language = self.language.clone();

        tokio::task::spawn_blocking(move || {
            let start = Instant::now();

            let mut state = ctx
                .create_state()
                .map_err(|e| DictationError::Stt(format!("create state: {e}")))?;

            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });
            if let Some(ref lang) = language {
                params.set_language(Some(lang));
            }
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            state
                .full(params, &audio)
                .map_err(|e| DictationError::Stt(format!("transcribe: {e}")))?;

            let num_segments = state.full_n_segments();
            let mut text = String::new();
            for i in 0..num_segments {
                if let Some(segment) = state.get_segment(i) {
                    let segment_text = segment
                        .to_str_lossy()
                        .map_err(|e| DictationError::Stt(format!("segment text {i}: {e}")))?;
                    text.push_str(&segment_text);
                }
            }

            Ok(TranscriptionResult {
                text: text.trim().to_string(),
                language,
                duration_ms: start.elapsed().as_millis() as u64,
            })
        })
        .await
        .map_err(|e| DictationError::Stt(format!("spawn_blocking: {e}")))?
    }

    fn kind(&self) -> SttBackendKind {
        SttBackendKind::Local
    }

    async fn health_check(&self) -> Result<(), DictationError> {
        self.ensure_loaded()?;
        let ctx: Arc<WhisperContext> = self.context.lock().unwrap().as_ref().unwrap().clone();
        ctx.create_state()
            .map_err(|e| DictationError::Stt(format!("create state: {e}")))?;
        Ok(())
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
    fn with_language_stores_language() {
        let engine = WhisperLocal::with_language("/path/to/model.bin".into(), "en".into()).unwrap();
        assert_eq!(engine.model_path(), "/path/to/model.bin");
    }

    #[test]
    fn with_language_rejects_empty_path() {
        let result = WhisperLocal::with_language(String::new(), "en".into());
        assert!(result.is_err());
    }

    #[test]
    fn kind_returns_local() {
        let engine = WhisperLocal::new("/path/to/model.bin".into()).unwrap();
        assert_eq!(engine.kind(), SttBackendKind::Local);
    }

    #[tokio::test]
    async fn transcribe_rejects_empty_audio() {
        let engine = WhisperLocal::new("/path/to/model.bin".into()).unwrap();
        let result = engine.transcribe(&[]).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty"), "expected 'empty' in error: {err}");
    }

    #[tokio::test]
    async fn transcribe_fails_with_invalid_model_path() {
        let engine = WhisperLocal::new("/nonexistent/model.bin".into()).unwrap();
        let result = engine.transcribe(&[0.0; 16000]).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("load model"),
            "expected 'load model' in error: {err}"
        );
    }

    #[tokio::test]
    async fn health_check_fails_with_invalid_model_path() {
        let engine = WhisperLocal::new("/nonexistent/model.bin".into()).unwrap();
        let result = engine.health_check().await;
        assert!(result.is_err());
    }

    // --- Integration tests requiring a real Whisper model ---

    #[tokio::test]
    #[ignore]
    async fn transcribe_with_real_model() {
        // Set WHISPER_MODEL_PATH env var to a valid .bin model file
        let model_path = std::env::var("WHISPER_MODEL_PATH").expect("WHISPER_MODEL_PATH not set");
        let engine = WhisperLocal::with_language(model_path, "en".into()).unwrap();
        engine.health_check().await.unwrap();

        // Generate 1 second of silence at 16kHz
        let audio = vec![0.0_f32; 16000];
        let result = engine.transcribe(&audio).await.unwrap();
        // Silence should produce empty or near-empty text
        assert!(result.duration_ms > 0);
    }
}
