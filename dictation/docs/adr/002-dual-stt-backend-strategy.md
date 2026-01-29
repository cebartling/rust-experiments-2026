# ADR-002: Dual STT Backend Strategy (Local + Cloud)

## Status

Accepted

## Context

Speech-to-text (STT) is the core functionality of the dictation application. We needed to choose between:

1. **Cloud-only**: OpenAI Whisper API, Google Speech-to-Text, Azure
2. **Local-only**: whisper.cpp, Vosk, Mozilla DeepSpeech
3. **Hybrid**: Support both local and cloud options

Key considerations:
- Privacy: Some users cannot send audio to cloud services
- Cost: Cloud APIs charge per-minute of audio
- Offline capability: Must work without internet
- Accuracy: Cloud models are typically more accurate
- Latency: Local inference can be faster for short audio
- Resource usage: Local models require CPU/GPU resources

## Decision

Implement dual STT backend strategy:

1. **Local backend**: `whisper-rs` (Rust bindings to whisper.cpp)
   - Default for privacy-conscious users
   - Offline capability
   - Free (after model download)

2. **Cloud backend**: OpenAI Whisper API
   - Higher accuracy on difficult audio
   - Lower resource usage
   - Per-minute pricing

Backend selection exposed in settings UI with clear privacy/cost tradeoffs.

## Consequences

### Positive

- **User choice**: Users select based on their privacy/cost/accuracy priorities
- **Flexibility**: Can fallback to cloud if local inference fails or is too slow
- **Privacy compliance**: Supports users with strict data residency requirements
- **Offline support**: Core functionality works without internet
- **Resource optimization**: Users on lower-end hardware can use cloud backend

### Negative

- **Increased complexity**: Must maintain two STT integrations
- **Testing burden**: Need to test both backends comprehensively
- **Dependency size**: whisper.cpp adds ~50MB to binary (model is separate)
- **Configuration complexity**: Users must understand tradeoffs to configure

### Implementation Details

- Abstract STT behind `SttEngine` trait
- Local: `whisper-rs` with quantized models for speed/size
- Cloud: `reqwest` for HTTP API calls to OpenAI
- Settings include: backend selection, API key (cloud), model size (local)
- Graceful error handling with fallback suggestions

### Risks

- **API changes**: OpenAI API changes could break cloud backend
- **Model compatibility**: whisper.cpp updates may break model loading
- **Cost surprises**: Users may not understand cloud API pricing

### Mitigations

- Version pin whisper-rs and test model compatibility
- Clear pricing display in UI before enabling cloud backend
- Usage tracking and warnings for cloud API consumption
