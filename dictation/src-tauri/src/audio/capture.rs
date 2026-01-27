use std::sync::{Arc, Mutex};

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
}

impl Default for AudioRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            state: RecorderState::Idle,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 0,
            channels: 0,
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
        self.state = RecorderState::Recording;
        // TODO: Phase 2 - start cpal audio stream
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<f32>, DictationError> {
        if self.state != RecorderState::Recording {
            return Err(DictationError::Audio("not recording".into()));
        }
        self.state = RecorderState::Idle;
        // TODO: Phase 2 - stop cpal audio stream
        let buffer = self.buffer.lock().unwrap().clone();
        Ok(buffer)
    }

    pub fn push_samples(&self, samples: &[f32]) {
        if self.state == RecorderState::Recording {
            self.buffer.lock().unwrap().extend_from_slice(samples);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
