# ADR-007: Frontend Framework Selection

## Status

Accepted

## Context

The Tauri frontend can use any web technology. We needed to choose a framework for the dictation app UI, which includes:

- Status indicator (idle/recording/transcribing)
- Settings panel (STT backend, API key, shortcuts, etc.)
- Transcription log (history of recent transcriptions)
- System tray menu integration

Options considered:

1. **Vanilla TypeScript + Vite**: No framework, direct DOM manipulation
2. **React**: Most popular, large ecosystem, JSX
3. **Vue**: Progressive, template-based, smaller bundle
4. **Svelte**: Compiled, minimal runtime, fast
5. **Solid**: Reactive, similar to React but faster

Requirements:
- Fast build times for development iteration
- TypeScript support
- Small bundle size (desktop app, size matters less than web)
- Simple state management (app state is simple)
- Team familiarity

## Decision

Use **Vite + TypeScript** with minimal framework dependencies:

- Vite for build tooling and dev server
- TypeScript for type safety
- Vanilla Web Components or lightweight library if needed
- Direct Tauri IPC calls via `@tauri-apps/api`
- CSS for styling (no CSS-in-JS complexity)

For this simple UI, a full framework is overkill. Keep it simple.

## Consequences

### Positive

- **Minimal dependencies**: Faster installs, smaller bundle
- **No framework lock-in**: Easy to migrate later if needed
- **Fast builds**: Vite provides excellent dev experience
- **Full control**: No framework magic, clear data flow
- **TypeScript benefits**: Type safety for Tauri commands and events
- **Lightweight**: Minimal runtime overhead
- **Simple state**: No need for Redux/Vuex/etc.

### Negative

- **More boilerplate**: No framework conveniences (reactivity, components)
- **Manual DOM updates**: Must handle updates explicitly
- **Less structure**: No enforced patterns for large growth
- **Smaller community**: Fewer examples of Tauri + vanilla setups

### Implementation Details

**Project Structure:**
```
src/
├── main.ts           # Entry point, Tauri setup
├── components/       # Reusable UI components
│   ├── StatusIndicator.ts
│   ├── SettingsPanel.ts
│   └── TranscriptionLog.ts
├── styles/           # CSS files
│   └── main.css
├── types/            # TypeScript type definitions
│   └── tauri.d.ts
└── utils/            # Helper functions
    └── tauri.ts      # Tauri IPC wrappers
```

**State Management:**
```typescript
// Simple global state object
const appState = {
  status: 'idle' as 'idle' | 'recording' | 'transcribing',
  settings: {} as AppSettings,
  transcriptions: [] as Transcription[],
};

// Update UI on state change
function updateUI() {
  document.querySelector('#status')!.textContent = appState.status;
  // ... other updates
}
```

**Tauri Integration:**
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Call Rust commands
await invoke('start_recording');

// Listen to events from Rust
await listen('transcription-complete', (event) => {
  appState.transcriptions.push(event.payload);
  updateUI();
});
```

### When to Revisit

Consider adding a framework if:
- UI complexity grows significantly (many interactive components)
- Team grows and needs enforced patterns
- Need advanced state management (time-travel, undo/redo)
- Want component marketplace/ecosystem

For MVP, vanilla approach is sufficient.

### Build Configuration

```javascript
// vite.config.ts
export default {
  clearScreen: false,
  server: {
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: false,
  },
};
```

### Developer Experience

- **Hot reload**: Vite provides instant feedback
- **TypeScript**: Catch errors at compile time
- **ESLint**: Enforce code quality
- **Prettier**: Consistent formatting

Keep tooling simple but effective.

### Risks

- **Scaling concerns**: Vanilla approach may not scale to complex UI
- **Code organization**: Requires discipline without framework structure

### Mitigations

- Clear coding conventions documented
- Modular component structure
- TypeScript for type safety and documentation
- Re-evaluate if UI complexity increases significantly
