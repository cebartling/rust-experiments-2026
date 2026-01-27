use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

use crate::error::DictationError;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Resamples multi-channel audio to 16kHz mono f32, as required by Whisper.
///
/// Steps:
/// 1. Mix down to mono (average all channels)
/// 2. Resample to 16kHz using rubato's sinc interpolation
pub fn resample_to_16khz_mono(
    input: &[f32],
    input_sample_rate: u32,
    input_channels: u16,
) -> Result<Vec<f32>, DictationError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    if input_channels == 0 {
        return Err(DictationError::Audio("channel count cannot be zero".into()));
    }
    if input_sample_rate == 0 {
        return Err(DictationError::Audio("sample rate cannot be zero".into()));
    }

    // Step 1: Mix down to mono if needed
    let mono = if input_channels == 1 {
        input.to_vec()
    } else {
        let channels = input_channels as usize;
        input
            .chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    // Step 2: Resample if needed
    if input_sample_rate == TARGET_SAMPLE_RATE {
        return Ok(mono);
    }

    resample_with_rubato(&mono, input_sample_rate)
}

fn resample_with_rubato(mono: &[f32], input_sample_rate: u32) -> Result<Vec<f32>, DictationError> {
    let ratio = TARGET_SAMPLE_RATE as f64 / input_sample_rate as f64;

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        oversampling_factor: 256,
        interpolation: SincInterpolationType::Linear,
        window: WindowFunction::BlackmanHarris2,
    };

    let chunk_size = 1024.max(mono.len().min(8192));

    let mut resampler = SincFixedIn::<f64>::new(ratio, 2.0, params, chunk_size, 1)
        .map_err(|e| DictationError::Audio(format!("create resampler: {e}")))?;

    // Convert f32 -> f64
    let input_f64: Vec<f64> = mono.iter().map(|&s| s as f64).collect();

    let frames_needed = resampler.input_frames_next();
    let mut output_f64: Vec<f64> = Vec::new();

    if input_f64.len() >= frames_needed {
        // Process full chunks
        let mut pos = 0;
        while pos + frames_needed <= input_f64.len() {
            let chunk = &input_f64[pos..pos + frames_needed];
            let result = resampler
                .process(&[chunk], None)
                .map_err(|e| DictationError::Audio(format!("resample: {e}")))?;
            output_f64.extend_from_slice(&result[0]);
            pos += frames_needed;
        }

        // Process remaining samples as partial
        if pos < input_f64.len() {
            let remaining = &input_f64[pos..];
            let result = resampler
                .process_partial(Some(&[remaining]), None)
                .map_err(|e| DictationError::Audio(format!("resample partial: {e}")))?;
            output_f64.extend_from_slice(&result[0]);
        }
    } else {
        // Input is shorter than one chunk — use process_partial directly
        let result = resampler
            .process_partial(Some(&[&input_f64]), None)
            .map_err(|e| DictationError::Audio(format!("resample partial: {e}")))?;
        output_f64.extend_from_slice(&result[0]);
    }

    // Convert f64 -> f32
    Ok(output_f64.iter().map(|&s| s as f32).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty_output() {
        let result = resample_to_16khz_mono(&[], 44100, 1).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn zero_channels_returns_error() {
        let result = resample_to_16khz_mono(&[1.0], 44100, 0);
        assert!(result.is_err());
    }

    #[test]
    fn zero_sample_rate_returns_error() {
        let result = resample_to_16khz_mono(&[1.0], 0, 1);
        assert!(result.is_err());
    }

    #[test]
    fn mono_16khz_passes_through() {
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let result = resample_to_16khz_mono(&input, 16_000, 1).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn stereo_mixes_to_mono() {
        // Stereo: L=1.0, R=0.0 -> mono = 0.5
        let input = vec![1.0, 0.0, 0.0, 1.0];
        let result = resample_to_16khz_mono(&input, 16_000, 2).unwrap();
        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.5).abs() < f32::EPSILON);
        assert!((result[1] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn downsampling_reduces_sample_count() {
        // 48kHz -> 16kHz should yield ~1/3 of the samples
        let input: Vec<f32> = (0..4800).map(|i| (i as f32 / 4800.0).sin()).collect();
        let result = resample_to_16khz_mono(&input, 48_000, 1).unwrap();
        let expected = (4800.0 * 16_000.0 / 48_000.0) as usize; // 1600
        // Allow some tolerance for resampler edge effects
        assert!(
            result.len().abs_diff(expected) < 50,
            "expected ~{expected}, got {}",
            result.len()
        );
    }

    #[test]
    fn upsampling_increases_sample_count() {
        // 8kHz -> 16kHz should roughly double the samples
        let input: Vec<f32> = (0..8000).map(|i| (i as f32 / 8000.0).sin()).collect();
        let result = resample_to_16khz_mono(&input, 8_000, 1).unwrap();
        let expected = (8000.0 * 16_000.0 / 8_000.0) as usize; // 16000
        // Sinc resampler adds latency padding; allow ~2% tolerance
        assert!(
            result.len().abs_diff(expected) < 400,
            "expected ~{expected}, got {}",
            result.len()
        );
    }

    #[test]
    fn resampled_signal_preserves_dc_level() {
        // Constant signal at 0.5 should remain ~0.5 after resampling
        let input = vec![0.5_f32; 4800];
        let result = resample_to_16khz_mono(&input, 48_000, 1).unwrap();
        // Skip first/last few samples (edge effects from sinc filter)
        let mid = &result[10..result.len().saturating_sub(10)];
        for &sample in mid {
            assert!((sample - 0.5).abs() < 0.01, "expected ~0.5, got {sample}");
        }
    }

    #[test]
    fn stereo_48khz_to_mono_16khz() {
        // Full pipeline: stereo 48kHz -> mono 16kHz
        let num_frames = 4800;
        let mut input = Vec::with_capacity(num_frames * 2);
        for i in 0..num_frames {
            let val = (i as f32 / num_frames as f32).sin();
            input.push(val); // L
            input.push(val); // R (same as L)
        }
        let result = resample_to_16khz_mono(&input, 48_000, 2).unwrap();
        let expected = (num_frames as f64 * 16_000.0 / 48_000.0) as usize;
        assert!(
            result.len().abs_diff(expected) < 50,
            "expected ~{expected}, got {}",
            result.len()
        );
    }

    #[test]
    fn small_input_resamples_correctly() {
        // Very small input (less than one chunk)
        let input = vec![0.0, 0.5, 1.0, 0.5, 0.0];
        let result = resample_to_16khz_mono(&input, 48_000, 1).unwrap();
        // 5 samples at 48kHz -> ~1-2 samples at 16kHz
        assert!(!result.is_empty());
    }

    #[test]
    fn common_sample_rate_44100() {
        // 44.1kHz is a very common sample rate
        let input: Vec<f32> = (0..4410).map(|i| (i as f32 / 4410.0).sin()).collect();
        let result = resample_to_16khz_mono(&input, 44_100, 1).unwrap();
        let expected = (4410.0 * 16_000.0 / 44_100.0) as usize; // 1600
        assert!(
            result.len().abs_diff(expected) < 50,
            "expected ~{expected}, got {}",
            result.len()
        );
    }
}
