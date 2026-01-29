# ADR-006: Settings Persistence Strategy

## Status

Accepted

## Context

The application needs to persist user settings across sessions:

- STT backend selection (local/cloud)
- OpenAI API key (if using cloud)
- Whisper model selection (if using local)
- Global keyboard shortcut
- Audio input device
- Typing speed for text injection
- UI preferences (theme, window position)

Storage options:

1. **Tauri plugin-store**: JSON-based key-value store
2. **SQL database**: SQLite via rusqlite
3. **Configuration file**: TOML/JSON in standard config directory
4. **OS-specific**: Registry (Windows), plist (macOS), dconf (Linux)

Requirements:
- Simple read/write operations
- Type-safe in Rust
- Secure storage for API keys
- Cross-platform consistency
- Migration support for schema changes

## Decision

Use `tauri-plugin-store` for settings persistence:

```rust
// Settings stored in JSON format
// Location: Platform-specific app data directory
{
  "stt_backend": "local",
  "openai_api_key": "sk-...",
  "whisper_model": "base",
  "keyboard_shortcut": "Ctrl+Shift+Space",
  "typing_delay_ms": 10
}
```

Architecture:
- `config` module defines `AppSettings` struct
- Default values via `Default` trait
- Serialize/deserialize via `serde`
- Tauri plugin-store handles file I/O
- In-memory cache in `AppState` for fast access

## Consequences

### Positive

- **Official Tauri integration**: Well-maintained, documented
- **Simple API**: Key-value operations, automatic serialization
- **Type safety**: Compile-time guarantees via serde
- **Atomic writes**: Plugin handles file locking and corruption prevention
- **Cross-platform**: Automatic platform-appropriate storage location
- **Hot reload**: Can watch for external file changes
- **Migration friendly**: Easy to add/remove fields with serde defaults

### Negative

- **JSON only**: No schema enforcement beyond serde
- **Not encrypted**: API keys stored in plaintext JSON
- **No validation**: Must validate in application code
- **File-based**: Concurrent access limited (though rare for settings)

### Implementation Details

**File Location:**
- macOS: `~/Library/Application Support/com.dictation.app/settings.json`
- Linux: `~/.config/dictation/settings.json`
- Windows: `%APPDATA%\dictation\settings.json`

**Schema:**
```rust
#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    // STT
    stt_backend: SttBackend,  // enum: Local | Cloud
    openai_api_key: Option<String>,
    whisper_model_size: ModelSize,  // enum: Tiny | Base | Small | Medium | Large

    // Input
    keyboard_shortcut: String,  // e.g., "Ctrl+Shift+Space"
    audio_device: Option<String>,  // device ID or default

    // Output
    typing_delay_ms: u32,  // delay between keystrokes
    use_clipboard_fallback: bool,

    // UI
    theme: Theme,  // enum: Light | Dark | System
    window_position: Option<(i32, i32)>,
}
```

**Access Pattern:**
```rust
// On startup: Load from disk into AppState
let settings = store.load().unwrap_or_default();

// During runtime: Read from AppState (fast)
let backend = app_state.settings.stt_backend;

// On settings change: Update both AppState and disk
app_state.settings.stt_backend = new_backend;
store.save(&app_state.settings)?;
```

### Security Considerations

**API Key Storage:**
- Current: Plaintext in JSON (acceptable for MVP)
- Future: OS keychain integration
  - macOS: Keychain Services
  - Linux: Secret Service API (libsecret)
  - Windows: Credential Manager

For MVP, document that API keys are stored unencrypted and users should use restricted-scope API keys.

### Migration Strategy

When schema changes:
```rust
// Old version missing new field
#[serde(default)]  // Uses Default trait
new_field: NewType,

// Deprecated field
#[serde(skip_serializing_if = "Option::is_none")]
old_field: Option<OldType>,
```

Serde's `default` attribute allows forward-compatible migrations.

### Validation

Settings validation on load:
```rust
impl AppSettings {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate keyboard shortcut format
        // Validate API key format (if present)
        // Validate typing delay in reasonable range
        // etc.
    }
}
```

### Risks

- **Corruption**: File corruption could lose all settings
- **Security**: Plaintext API keys visible to local users
- **Migration failures**: Schema changes could break loading

### Mitigations

- Backup file created before writes
- Fallback to defaults on corruption
- Clear error messages for validation failures
- Future: Implement OS keychain for sensitive data
- Documentation: Advise users on API key security
