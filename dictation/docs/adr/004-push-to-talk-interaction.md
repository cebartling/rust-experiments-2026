# ADR-004: Push-to-Talk Interaction Model

## Status

Accepted

## Context

Dictation applications typically use one of three interaction models:

1. **Push-to-talk**: Hold key to record, release to transcribe
2. **Click-to-toggle**: Click once to start, click again to stop
3. **Voice activation**: Automatically detect speech and record

Each has different UX and technical implications:

| Model | Pros | Cons |
|-------|------|------|
| Push-to-talk | Precise control, no false triggers | Requires holding key, hand strain |
| Click-to-toggle | Hands-free after activation | Accidental background noise, unclear when recording |
| Voice activation | Fully hands-free | False positives, privacy concerns, complexity |

## Decision

Implement **push-to-talk** as the primary interaction model:

- User holds a global keyboard shortcut (default: Ctrl+Shift+Space)
- Audio recording starts immediately on key press
- Recording continues while key is held
- On key release, audio is sent to STT backend
- Transcribed text is injected at cursor position

## Consequences

### Positive

- **Predictable**: User has complete control over when recording happens
- **No false triggers**: Only records when explicitly activated
- **Privacy-friendly**: User knows exactly when microphone is active
- **Simple implementation**: Clear state machine (idle → recording → transcribing → idle)
- **Low resource usage**: No continuous audio processing
- **Immediate feedback**: Visual indicator shows recording state

### Negative

- **Hand strain**: Long dictation sessions require holding key
- **Interruptions**: User must release key between phrases
- **Learning curve**: Users must adapt to push-to-talk workflow
- **Limited accessibility**: Difficult for users with motor impairments

### Implementation Details

**State Machine:**
```
Idle → (key press) → Recording → (key release) → Transcribing → (result) → Idle
```

**Components:**
- `tauri-plugin-global-shortcut`: Register global keyboard shortcut
- `hotkey` module: State machine and event handling
- `audio` module: Start/stop recording based on state
- Visual indicator: Frontend shows current state (idle/recording/transcribing)

**User Experience:**
1. User positions cursor in target application
2. Holds shortcut key
3. Speaks while key is held
4. Visual feedback shows recording
5. Releases key
6. Visual feedback shows "transcribing..."
7. Text appears at cursor position

### Future Enhancements

Could add alternative modes in settings:
- Click-to-toggle mode for accessibility
- Configurable keyboard shortcut
- Foot pedal support via USB HID
- Voice activation as opt-in experimental feature

### Risks

- **Shortcut conflicts**: May conflict with other applications
- **Missed releases**: Key release events might be lost in some scenarios
- **Cross-platform differences**: Keyboard event handling varies by OS

### Mitigations

- Configurable keyboard shortcut with conflict detection
- Timeout mechanism: Auto-stop recording after N seconds
- Fallback: Allow manual stop via system tray menu
- Clear documentation of shortcut customization
