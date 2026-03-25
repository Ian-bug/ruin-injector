use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Application configuration structure
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub dll_path: Option<String>,
    pub last_process: Option<String>,
    pub auto_inject: bool,
}

impl Config {
    /// Get the path to the configuration file
    fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                // Fallback to current directory if config_dir fails
                // This is a last resort and may have permission issues
                PathBuf::from(".")
            })
            .join("ruin-injector")
            .join("config.json")
    }

    /// Load configuration from file or return default
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if let Ok(config_str) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&config_str) {
                return config;
            }
        }

        Config::default()
    }

    /// Save configuration to file with atomic write to prevent corruption
    /// Writes to temp file first, then renames to final location
    pub fn save(&self) -> Result<(), io::Error> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_str = serde_json::to_string_pretty(self)?;

        // Atomic write: write to temp file, then rename
        let temp_path = config_path.with_extension("tmp");
        fs::write(&temp_path, config_str)?;
        fs::rename(&temp_path, &config_path)?;

        Ok(())
    }

    /// Save configuration and return error message if failed (for logging in UI)
    pub fn save_with_error_message(&self) -> Option<String> {
        match self.save() {
            Ok(()) => None,
            Err(e) => Some(format!("Failed to save config: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.dll_path.is_none());
        assert!(config.last_process.is_none());
        assert!(!config.auto_inject);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            dll_path: Some("C:\\test\\test.dll".to_string()),
            last_process: Some("notepad.exe".to_string()),
            auto_inject: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.dll_path, deserialized.dll_path);
        assert_eq!(config.last_process, deserialized.last_process);
        assert_eq!(config.auto_inject, deserialized.auto_inject);
    }

    #[test]
    fn test_config_save_load() {
        let config = Config {
            dll_path: Some("C:\\test\\test.dll".to_string()),
            last_process: Some("notepad.exe".to_string()),
            auto_inject: true,
        };

        // Save to a temporary path
        let temp_path = std::env::temp_dir().join("test_config.json");
        let _ = std::fs::remove_file(&temp_path);

        // Manually save to temp path for testing
        let json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(&temp_path, json).unwrap();

        // Load from the temp path directly
        let json_content = std::fs::read_to_string(&temp_path).unwrap();
        let loaded: Config = serde_json::from_str(&json_content).unwrap();

        assert_eq!(config.dll_path, loaded.dll_path);
        assert_eq!(config.last_process, loaded.last_process);
        assert_eq!(config.auto_inject, loaded.auto_inject);

        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_config_invalid_json() {
        // Invalid JSON should fail to parse
        let invalid_json = "{ invalid json }";
        let result: Result<Config, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Invalid JSON should fail to parse");
    }

    #[test]
    fn test_config_partial_json() {
        // Partial JSON with only some fields
        let partial_json = r#"{"dll_path":"C:\\test\\test.dll","auto_inject":true}"#;
        let config: Config = serde_json::from_str(partial_json).unwrap();
        assert_eq!(config.dll_path, Some("C:\\test\\test.dll".to_string()));
        assert!(config.last_process.is_none());
        assert!(config.auto_inject);
    }
}
