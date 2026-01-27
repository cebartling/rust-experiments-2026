use crate::error::DictationError;

pub struct TextInjector;

impl TextInjector {
    pub fn new() -> Result<Self, DictationError> {
        // TODO: Phase 4 - initialize enigo
        Ok(Self)
    }

    pub fn inject(&self, text: &str) -> Result<(), DictationError> {
        if text.is_empty() {
            return Ok(());
        }
        // TODO: Phase 4 - use enigo to type text into focused application
        Err(DictationError::Injection(
            "text injection not yet implemented".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_injector() {
        let injector = TextInjector::new();
        assert!(injector.is_ok());
    }

    #[test]
    fn inject_empty_string_succeeds() {
        let injector = TextInjector::new().unwrap();
        let result = injector.inject("");
        assert!(result.is_ok());
    }

    #[test]
    fn inject_text_returns_not_implemented() {
        let injector = TextInjector::new().unwrap();
        let result = injector.inject("hello world");
        assert!(result.is_err());
    }
}
