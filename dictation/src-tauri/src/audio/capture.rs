use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig, SupportedStreamConfig};

use crate::error::DictationError;

#[derive(Debug, Clone, PartialEq)]
pub enum RecorderState {
    Idle,
    Recording,
}

pub struct AudioRecorder {
    state: RecorderState,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    stream: Option<Stream>,
    device: Option<Device>,
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioRecorder {
    /// Creates a recorder without binding to any audio device.
    /// Useful for testing the state machine without hardware.
    pub fn new() -> Self {
        Self {
            state: RecorderState::Idle,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 0,
            channels: 0,
            stream: None,
            device: None,
        }
    }

    /// Creates a recorder bound to the system's default input device.
    pub fn with_default_device() -> Result<Self, DictationError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| DictationError::Audio("no input device available".into()))?;
        Ok(Self {
            state: RecorderState::Idle,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 0,
            channels: 0,
            stream: None,
            device: Some(device),
        })
    }

    /// Creates a recorder bound to a specific device.
    pub fn with_device(device: Device) -> Self {
        Self {
            state: RecorderState::Idle,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 0,
            channels: 0,
            stream: None,
            device: Some(device),
        }
    }

    pub fn state(&self) -> &RecorderState {
        &self.state
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn start(&mut self) -> Result<(), DictationError> {
        if self.state == RecorderState::Recording {
            return Err(DictationError::Audio("already recording".into()));
        }
        self.buffer.lock().unwrap().clear();

        if let Some(device) = &self.device {
            let supported_config = device
                .default_input_config()
                .map_err(|e| DictationError::Audio(format!("default input config: {e}")))?;

            self.sample_rate = supported_config.sample_rate().0;
            self.channels = supported_config.channels();

            let stream = build_input_stream(device, &supported_config, self.buffer.clone())?;
            stream
                .play()
                .map_err(|e| DictationError::Audio(format!("play stream: {e}")))?;
            self.stream = Some(stream);
        }

        self.state = RecorderState::Recording;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<f32>, DictationError> {
        if self.state != RecorderState::Recording {
            return Err(DictationError::Audio("not recording".into()));
        }
        // Drop the stream to stop recording
        self.stream = None;
        self.state = RecorderState::Idle;
        let buffer = self.buffer.lock().unwrap().clone();
        Ok(buffer)
    }

    /// Manually push samples into the buffer (used for testing without hardware).
    pub fn push_samples(&self, samples: &[f32]) {
        if self.state == RecorderState::Recording {
            self.buffer.lock().unwrap().extend_from_slice(samples);
        }
    }
}

/// Builds a cpal input stream that pushes f32 samples into the shared buffer,
/// handling sample format conversion for the device's native format.
fn build_input_stream(
    device: &Device,
    supported_config: &SupportedStreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<Stream, DictationError> {
    let config: StreamConfig = supported_config.clone().into();
    let err_fn = |err: cpal::StreamError| {
        eprintln!("audio stream error: {err}");
    };

    let stream = match supported_config.sample_format() {
        SampleFormat::F32 => device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    buffer.lock().unwrap().extend_from_slice(data);
                },
                err_fn,
                None,
            )
            .map_err(|e| DictationError::Audio(format!("build stream: {e}")))?,
        SampleFormat::I16 => device
            .build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let floats: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    buffer.lock().unwrap().extend_from_slice(&floats);
                },
                err_fn,
                None,
            )
            .map_err(|e| DictationError::Audio(format!("build stream: {e}")))?,
        SampleFormat::U16 => device
            .build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let floats: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                        .collect();
                    buffer.lock().unwrap().extend_from_slice(&floats);
                },
                err_fn,
                None,
            )
            .map_err(|e| DictationError::Audio(format!("build stream: {e}")))?,
        format => {
            return Err(DictationError::Audio(format!(
                "unsupported sample format: {format:?}"
            )));
        }
    };

    Ok(stream)
}

/// Lists all available audio input device names.
pub fn list_input_devices() -> Result<Vec<String>, DictationError> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| DictationError::Audio(format!("enumerate devices: {e}")))?;

    let names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- State machine tests (no hardware needed) ---

    #[test]
    fn new_recorder_is_idle() {
        let recorder = AudioRecorder::new();
        assert_eq!(*recorder.state(), RecorderState::Idle);
    }

    #[test]
    fn start_transitions_to_recording() {
        let mut recorder = AudioRecorder::new();
        recorder.start().unwrap();
        assert_eq!(*recorder.state(), RecorderState::Recording);
    }

    #[test]
    fn start_while_recording_returns_error() {
        let mut recorder = AudioRecorder::new();
        recorder.start().unwrap();
        let result = recorder.start();
        assert!(result.is_err());
    }

    #[test]
    fn stop_returns_buffer_and_transitions_to_idle() {
        let mut recorder = AudioRecorder::new();
        recorder.start().unwrap();
        recorder.push_samples(&[0.1, 0.2, 0.3]);
        let buffer = recorder.stop().unwrap();
        assert_eq!(buffer, vec![0.1, 0.2, 0.3]);
        assert_eq!(*recorder.state(), RecorderState::Idle);
    }

    #[test]
    fn stop_while_idle_returns_error() {
        let mut recorder = AudioRecorder::new();
        let result = recorder.stop();
        assert!(result.is_err());
    }

    #[test]
    fn push_samples_only_works_while_recording() {
        let mut recorder = AudioRecorder::new();
        recorder.push_samples(&[1.0, 2.0]);
        // Not recording, so buffer should be empty
        recorder.start().unwrap();
        let buffer = recorder.stop().unwrap();
        assert!(buffer.is_empty());
    }

    #[test]
    fn push_samples_accumulates_during_recording() {
        let mut recorder = AudioRecorder::new();
        recorder.start().unwrap();
        recorder.push_samples(&[0.1, 0.2]);
        recorder.push_samples(&[0.3, 0.4]);
        let buffer = recorder.stop().unwrap();
        assert_eq!(buffer, vec![0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn start_clears_previous_buffer() {
        let mut recorder = AudioRecorder::new();
        recorder.start().unwrap();
        recorder.push_samples(&[1.0, 2.0]);
        recorder.stop().unwrap();
        recorder.start().unwrap();
        let buffer = recorder.stop().unwrap();
        assert!(buffer.is_empty());
    }

    #[test]
    fn sample_rate_and_channels_default_to_zero_without_device() {
        let recorder = AudioRecorder::new();
        assert_eq!(recorder.sample_rate(), 0);
        assert_eq!(recorder.channels(), 0);
    }

    // --- Hardware integration tests (require a microphone) ---

    #[test]
    #[ignore]
    fn with_default_device_succeeds_when_mic_present() {
        let recorder = AudioRecorder::with_default_device();
        assert!(recorder.is_ok());
    }

    #[test]
    #[ignore]
    fn start_with_device_populates_sample_rate_and_channels() {
        let mut recorder = AudioRecorder::with_default_device().unwrap();
        recorder.start().unwrap();
        assert!(recorder.sample_rate() > 0);
        assert!(recorder.channels() > 0);
        recorder.stop().unwrap();
    }

    #[test]
    #[ignore]
    fn list_input_devices_returns_at_least_one() {
        let devices = list_input_devices().unwrap();
        assert!(!devices.is_empty());
    }
}
