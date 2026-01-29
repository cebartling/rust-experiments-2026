# ADR-003: Audio Pipeline Architecture (cpal + rubato)

## Status

Accepted

## Context

The audio pipeline must:
1. Capture microphone input at native sample rate
2. Resample to 16kHz mono (required by Whisper models)
3. Buffer audio during push-to-talk sessions
4. Minimize latency and CPU overhead
5. Work cross-platform (macOS, Linux, Windows)

Alternatives considered:

**Audio Capture Libraries:**
- **cpal**: Cross-platform, low-level, Rust-native
- **rodio**: Higher-level but focused on playback
- **portaudio**: C library with Rust bindings, heavier

**Resampling Libraries:**
- **rubato**: Pure Rust, high-quality resampling
- **samplerate**: Rust bindings to libsamplerate (C library)
- **dasp**: Rust audio processing, but heavier than needed

## Decision

Use `cpal` for audio capture and `rubato` for resampling:

```
Microphone → cpal → Buffer → rubato → 16kHz mono → STT
```

Architecture:
- `cpal::Stream` captures microphone at native sample rate
- Circular buffer stores incoming audio
- On push-to-talk release, buffer contents resampled via `rubato`
- Resampled audio sent to STT backend

## Consequences

### Positive

- **Cross-platform**: cpal supports all target platforms
- **Low overhead**: Direct access to audio APIs (CoreAudio, ALSA, WASAPI)
- **Pure Rust**: No C dependencies, easier to build and distribute
- **High quality**: rubato provides excellent resampling quality
- **Minimal latency**: Direct streaming with minimal buffering
- **Flexible**: Can adapt to any input sample rate

### Negative

- **Low-level complexity**: Must handle sample rate conversions manually
- **Buffer management**: Need to implement circular buffer logic
- **Error handling**: Must handle device changes, permission errors
- **Testing difficulty**: Audio device testing requires hardware/mocking

### Implementation Details

- Default input: System default microphone
- Buffer size: 30 seconds max (configurable)
- Resampling quality: High quality mode (configurable for performance)
- Channel conversion: Mix stereo to mono by averaging
- Sample format: f32 normalized samples

### Technical Choices

- **Sample rate**: 16kHz output (Whisper requirement)
- **Bit depth**: 32-bit float internally, convert to 16-bit PCM for Whisper
- **Channels**: Convert all input to mono
- **Buffer implementation**: Ring buffer with wrap-around

### Risks

- **Buffer overflow**: If push-to-talk held longer than buffer size
- **Device errors**: Microphone unplugged during capture
- **Permission errors**: OS denies microphone access

### Mitigations

- Clear buffer size limit with UI warning when approaching limit
- Graceful error handling with user-friendly messages
- Permission request on first use with helpful error messages
- Device change detection and automatic reconnection
