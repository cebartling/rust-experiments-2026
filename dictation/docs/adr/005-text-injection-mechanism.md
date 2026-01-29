# ADR-005: Text Injection Mechanism

## Status

Accepted

## Context

After transcription, we need to insert the text at the user's cursor position in the active application. Platform options:

**macOS:**
- Accessibility API (private, requires permissions)
- Keyboard simulation (CGEventPost)
- Pasteboard + Cmd+V
- AppleScript

**Linux:**
- X11: XTestFakeKeyEvent, xdotool
- Wayland: Limited/no programmatic input (security model)
- Clipboard + Ctrl+V

**Windows:**
- SendInput API
- Clipboard + Ctrl+V
- UI Automation

**Cross-platform libraries:**
- **enigo**: Keyboard/mouse simulation, cross-platform
- **autopilot-rs**: Similar but less maintained
- **rdev**: Input listening, limited output

## Decision

Use `enigo` library for keyboard simulation to inject text:

- Simulate typing each character via OS keyboard APIs
- Fallback to clipboard paste (Ctrl+V / Cmd+V) for long text
- Handle special characters and Unicode properly

## Consequences

### Positive

- **Cross-platform**: Single API for macOS, Linux (X11), Windows
- **Direct injection**: Text appears at cursor position, no manual paste
- **No clipboard pollution**: Preserves user's clipboard content (when typing)
- **Rust-native**: Pure Rust implementation, no C dependencies
- **Character preservation**: Handles Unicode and special characters

### Negative

- **Speed limits**: Typing simulation slower than direct text insertion
- **Focus issues**: Target window must remain focused during injection
- **Wayland limitations**: Limited support on Linux Wayland
- **Permission requirements**: Requires accessibility permissions on macOS
- **App compatibility**: Some apps may ignore simulated keyboard events

### Implementation Details

**Strategy:**
```rust
if text.len() < 1000 {
    // Type character by character
    enigo.text(&text);
} else {
    // Fallback: clipboard paste for long text
    clipboard.set_text(&text);
    enigo.key_sequence(&platform_paste_shortcut());
}
```

**Considerations:**
- Typing delay: Small delay between characters (configurable)
- Special characters: Map to proper key combinations
- Platform paste shortcuts: Cmd+V (macOS), Ctrl+V (Linux/Windows)
- Error handling: Detect and report injection failures

### Platform-Specific Notes

**macOS:**
- Requires "Accessibility" permission in System Preferences
- Must prompt user to grant permission on first use
- Works reliably across most applications

**Linux:**
- X11: Works well with xdotool backend
- Wayland: Limited functionality, clipboard paste only
- Needs X11 libraries installed (libxdo-dev)

**Windows:**
- Uses SendInput API
- Works across most applications
- May trigger anti-cheat in some games

### Risks

- **Injection failures**: Some apps ignore simulated input
- **Race conditions**: Text injected before target app is ready
- **Unicode issues**: Complex emoji or RTL text may fail
- **Security software**: AV/EDR may block keyboard simulation

### Mitigations

- Clear permission request UI with setup instructions
- Validation mode: Test injection before first use
- Retry logic with exponential backoff
- Fallback to clipboard paste with user notification
- Platform-specific handling for known problematic apps
- Settings to adjust typing speed and retry behavior

### Alternative Considered

**Clipboard-only approach:**
- Pro: More reliable, works on Wayland
- Con: Pollutes clipboard, requires manual paste
- Con: Poor UX (user must Cmd+V manually)

Rejected because it breaks the seamless dictation experience.
