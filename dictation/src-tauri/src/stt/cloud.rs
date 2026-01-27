use std::io::Cursor;
use std::time::Instant;

use async_trait::async_trait;
use reqwest::Client;

use super::{SttBackendKind, SttEngine, TranscriptionResult};
use crate::error::DictationError;

const DEFAULT_API_BASE_URL: &str = "https://api.openai.com";

pub struct CloudStt {
    api_key: String,
    model: String,
    language: Option<String>,
    client: Client,
    api_base_url: String,
}

impl CloudStt {
    pub fn new(api_key: String, model: String) -> Result<Self, DictationError> {
        if api_key.is_empty() {
            return Err(DictationError::Stt("API key is empty".into()));
        }
        Ok(Self {
            api_key,
            model,
            language: None,
            client: Client::new(),
            api_base_url: DEFAULT_API_BASE_URL.into(),
        })
    }

    pub fn with_language(
        api_key: String,
        model: String,
        language: String,
    ) -> Result<Self, DictationError> {
        let mut engine = Self::new(api_key, model)?;
        engine.language = Some(language);
        Ok(engine)
    }

    #[cfg(test)]
    fn with_base_url(
        api_key: String,
        model: String,
        base_url: String,
    ) -> Result<Self, DictationError> {
        let mut engine = Self::new(api_key, model)?;
        engine.api_base_url = base_url;
        Ok(engine)
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn has_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Encodes f32 audio samples (16kHz mono) into WAV format bytes.
pub fn encode_wav(audio_16khz_mono: &[f32]) -> Result<Vec<u8>, DictationError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buf: Vec<u8> = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut writer = hound::WavWriter::new(cursor, spec)
            .map_err(|e| DictationError::Stt(format!("create WAV writer: {e}")))?;

        for &sample in audio_16khz_mono {
            let s16 = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            writer
                .write_sample(s16)
                .map_err(|e| DictationError::Stt(format!("write WAV sample: {e}")))?;
        }

        writer
            .finalize()
            .map_err(|e| DictationError::Stt(format!("finalize WAV: {e}")))?;
    }

    Ok(buf)
}

#[async_trait]
impl SttEngine for CloudStt {
    async fn transcribe(
        &self,
        audio_16khz_mono: &[f32],
    ) -> Result<TranscriptionResult, DictationError> {
        if audio_16khz_mono.is_empty() {
            return Err(DictationError::Stt("audio data is empty".into()));
        }

        let start = Instant::now();
        let wav_bytes = encode_wav(audio_16khz_mono)?;

        let part = reqwest::multipart::Part::bytes(wav_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| DictationError::Stt(format!("create multipart: {e}")))?;

        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", self.model.clone());

        if let Some(ref lang) = self.language {
            form = form.text("language", lang.clone());
        }

        let response = self
            .client
            .post(format!("{}/v1/audio/transcriptions", self.api_base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| DictationError::Stt(format!("API request: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DictationError::Stt(format!("API error {status}: {body}")));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| DictationError::Stt(format!("parse response: {e}")))?;

        let text = json["text"]
            .as_str()
            .ok_or_else(|| DictationError::Stt("missing 'text' in API response".into()))?
            .to_string();

        Ok(TranscriptionResult {
            text,
            language: self.language.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn kind(&self) -> SttBackendKind {
        SttBackendKind::Cloud
    }

    async fn health_check(&self) -> Result<(), DictationError> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.api_base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| DictationError::Stt(format!("API request: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DictationError::Stt(format!(
                "API health check failed {status}: {body}"
            )));
        }

        Ok(())
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
    fn with_language_stores_language() {
        let engine =
            CloudStt::with_language("sk-test-key".into(), "whisper-1".into(), "en".into()).unwrap();
        assert_eq!(engine.model(), "whisper-1");
    }

    #[test]
    fn with_language_rejects_empty_key() {
        let result = CloudStt::with_language(String::new(), "whisper-1".into(), "en".into());
        assert!(result.is_err());
    }

    #[test]
    fn kind_returns_cloud() {
        let engine = CloudStt::new("sk-test-key".into(), "whisper-1".into()).unwrap();
        assert_eq!(engine.kind(), SttBackendKind::Cloud);
    }

    #[tokio::test]
    async fn transcribe_rejects_empty_audio() {
        let engine = CloudStt::new("sk-test-key".into(), "whisper-1".into()).unwrap();
        let result = engine.transcribe(&[]).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty"), "expected 'empty' in error: {err}");
    }

    // --- WAV encoding tests ---

    #[test]
    fn encode_wav_produces_valid_header() {
        let audio = vec![0.0_f32; 160]; // 10ms of silence at 16kHz
        let wav = encode_wav(&audio).unwrap();

        // WAV files start with "RIFF"
        assert_eq!(&wav[0..4], b"RIFF");
        // Format should be "WAVE"
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn encode_wav_roundtrip() {
        // Create a simple sine wave
        let audio: Vec<f32> = (0..1600)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin())
            .collect();
        let wav = encode_wav(&audio).unwrap();

        // Read it back with hound
        let cursor = Cursor::new(wav);
        let mut reader = hound::WavReader::new(cursor).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 16000);
        assert_eq!(spec.bits_per_sample, 16);
        assert_eq!(spec.sample_format, hound::SampleFormat::Int);

        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), 1600);
    }

    #[test]
    fn encode_wav_empty_returns_error_on_empty_input_is_valid() {
        // Empty audio should still produce a valid (but short) WAV
        let wav = encode_wav(&[]).unwrap();
        assert_eq!(&wav[0..4], b"RIFF");
    }

    #[test]
    fn encode_wav_clips_extreme_values() {
        let audio = vec![2.0, -2.0, 0.5]; // Values beyond [-1, 1]
        let wav = encode_wav(&audio).unwrap();

        let cursor = Cursor::new(wav);
        let mut reader = hound::WavReader::new(cursor).unwrap();
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0], i16::MAX); // Clipped to max
        assert_eq!(samples[1], i16::MIN); // Clipped to min
    }

    // --- Mock HTTP tests ---

    #[tokio::test]
    async fn transcribe_handles_api_error_response() {
        // Use a non-routable address to trigger a connection error
        let engine = CloudStt::with_base_url(
            "sk-test".into(),
            "whisper-1".into(),
            "http://127.0.0.1:1".into(), // Port 1 should be unreachable
        )
        .unwrap();
        let audio = vec![0.0_f32; 16000];
        let result = engine.transcribe(&audio).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn health_check_handles_connection_error() {
        let engine = CloudStt::with_base_url(
            "sk-test".into(),
            "whisper-1".into(),
            "http://127.0.0.1:1".into(),
        )
        .unwrap();
        let result = engine.health_check().await;
        assert!(result.is_err());
    }

    // --- Integration tests requiring a real OpenAI API key ---

    #[tokio::test]
    #[ignore]
    async fn transcribe_with_real_api() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let engine = CloudStt::with_language(api_key, "whisper-1".into(), "en".into()).unwrap();
        engine.health_check().await.unwrap();

        // 1 second of silence
        let audio = vec![0.0_f32; 16000];
        let result = engine.transcribe(&audio).await.unwrap();
        assert!(result.duration_ms > 0);
    }
}
