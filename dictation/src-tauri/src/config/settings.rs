use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub hotkey: String,
    pub stt_backend: SttBackendChoice,
    pub local_model_path: String,
    pub local_model_size: ModelSize,
    pub cloud_provider: String,
    pub cloud_api_key: String,
    pub cloud_model: String,
    pub language: String,
    pub auto_inject: bool,
    pub notifications: bool,
    pub start_minimized: bool,
    pub launch_at_startup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SttBackendChoice {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+Space".into(),
            stt_backend: SttBackendChoice::Local,
            local_model_path: String::new(),
            local_model_size: ModelSize::Base,
            cloud_provider: "openai".into(),
            cloud_api_key: String::new(),
            cloud_model: "whisper-1".into(),
            language: "en".into(),
            auto_inject: true,
            notifications: true,
            start_minimized: false,
            launch_at_startup: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_expected_hotkey() {
        let settings = AppSettings::default();
        assert_eq!(settings.hotkey, "Ctrl+Shift+Space");
    }

    #[test]
    fn default_settings_use_local_backend() {
        let settings = AppSettings::default();
        assert_eq!(settings.stt_backend, SttBackendChoice::Local);
    }

    #[test]
    fn default_settings_use_base_model() {
        let settings = AppSettings::default();
        assert_eq!(settings.local_model_size, ModelSize::Base);
    }

    #[test]
    fn default_settings_use_english() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn default_settings_enable_auto_inject() {
        let settings = AppSettings::default();
        assert!(settings.auto_inject);
    }

    #[test]
    fn settings_serialize_roundtrip() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, deserialized);
    }

    #[test]
    fn stt_backend_choice_serializes_lowercase() {
        let local = serde_json::to_string(&SttBackendChoice::Local).unwrap();
        assert_eq!(local, "\"local\"");
        let cloud = serde_json::to_string(&SttBackendChoice::Cloud).unwrap();
        assert_eq!(cloud, "\"cloud\"");
    }

    #[test]
    fn settings_serialize_camel_case_field_names() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"sttBackend\""));
        assert!(json.contains("\"localModelPath\""));
        assert!(json.contains("\"localModelSize\""));
        assert!(json.contains("\"cloudApiKey\""));
        assert!(json.contains("\"cloudModel\""));
        assert!(json.contains("\"autoInject\""));
        assert!(json.contains("\"startMinimized\""));
        assert!(json.contains("\"launchAtStartup\""));
        // Should NOT contain snake_case versions
        assert!(!json.contains("\"stt_backend\""));
        assert!(!json.contains("\"local_model_path\""));
        assert!(!json.contains("\"auto_inject\""));
    }

    #[test]
    fn model_size_serializes_lowercase() {
        let tiny = serde_json::to_string(&ModelSize::Tiny).unwrap();
        assert_eq!(tiny, "\"tiny\"");
        let large = serde_json::to_string(&ModelSize::Large).unwrap();
        assert_eq!(large, "\"large\"");
    }
}
