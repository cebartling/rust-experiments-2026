use crate::error::DictationError;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

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

    // TODO: Phase 2 - use rubato for high-quality resampling
    // For now, use simple linear interpolation as a placeholder
    let ratio = TARGET_SAMPLE_RATE as f64 / input_sample_rate as f64;
    let output_len = (mono.len() as f64 * ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos.floor() as usize;
        let frac = src_pos - src_idx as f64;

        let sample = if src_idx + 1 < mono.len() {
            mono[src_idx] * (1.0 - frac as f32) + mono[src_idx + 1] * frac as f32
        } else if src_idx < mono.len() {
            mono[src_idx]
        } else {
            0.0
        };
        output.push(sample);
    }

    Ok(output)
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
        // 32kHz -> 16kHz should roughly halve the samples
        let input: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
        let result = resample_to_16khz_mono(&input, 32_000, 1).unwrap();
        assert!(result.len() <= 51); // approximately half
        assert!(result.len() >= 49);
    }

    #[test]
    fn upsampling_increases_sample_count() {
        // 8kHz -> 16kHz should roughly double the samples
        let input: Vec<f32> = (0..100).map(|i| (i as f32) / 100.0).collect();
        let result = resample_to_16khz_mono(&input, 8_000, 1).unwrap();
        assert!(result.len() >= 199);
        assert!(result.len() <= 201);
    }
}
