use std::sync::Mutex;

use enigo::{Enigo, Keyboard, Settings};

use crate::error::DictationError;

/// Injects text into the currently focused application using enigo
/// for cross-platform keyboard simulation.
///
/// The display server connection is established lazily on the first
/// call to `inject()`, so construction always succeeds regardless
/// of whether a display server is available.
pub struct TextInjector {
    enigo: Mutex<Option<Enigo>>,
}

impl Default for TextInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInjector {
    pub fn new() -> Self {
        Self {
            enigo: Mutex::new(None),
        }
    }

    /// Types the given text into the currently focused application.
    ///
    /// Returns `Ok(())` for empty strings without initializing enigo.
    /// On first non-empty call, establishes a connection to the display
    /// server (X11/Wayland on Linux, native APIs on macOS/Windows).
    pub fn inject(&self, text: &str) -> Result<(), DictationError> {
        if text.is_empty() {
            return Ok(());
        }

        let mut guard = self.enigo.lock().unwrap();
        if guard.is_none() {
            let enigo = Enigo::new(&Settings::default())
                .map_err(|e| DictationError::Injection(format!("initialize keyboard: {e}")))?;
            *guard = Some(enigo);
        }

        guard
            .as_mut()
            .unwrap()
            .text(text)
            .map_err(|e| DictationError::Injection(format!("type text: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_injector() {
        let _injector = TextInjector::new();
    }

    #[test]
    fn default_creates_injector() {
        let _injector = TextInjector::default();
    }

    #[test]
    fn inject_empty_string_succeeds() {
        let injector = TextInjector::new();
        let result = injector.inject("");
        assert!(result.is_ok());
    }

    #[test]
    fn inject_does_not_panic() {
        // On headless systems, inject may fail due to no display server.
        // On desktop systems, it will type into the focused app.
        // Either way, it must not panic.
        let injector = TextInjector::new();
        let _ = injector.inject("test");
    }

    // --- Integration tests requiring a desktop environment ---

    #[test]
    #[ignore]
    fn inject_types_text_into_focused_app() {
        let injector = TextInjector::new();
        injector.inject("Hello from dictation!").unwrap();
    }

    #[test]
    #[ignore]
    fn inject_handles_unicode() {
        let injector = TextInjector::new();
        injector.inject("Caf\u{00e9} na\u{00ef}ve").unwrap();
    }

    #[test]
    #[ignore]
    fn inject_handles_multiline() {
        let injector = TextInjector::new();
        injector.inject("line one\nline two\nline three").unwrap();
    }
}
