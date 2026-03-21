use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub dll_path: Option<String>,
    pub last_process: Option<String>,
    pub auto_inject: bool,
}

impl Config {
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

    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if let Ok(config_str) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&config_str) {
                return config;
            }
        }

        Config::default()
    }

    /// Save config with atomic write to prevent corruption
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

    /// Save config and return error message if failed (for logging in UI)
    pub fn save_with_error_message(&self) -> Option<String> {
        match self.save() {
            Ok(()) => None,
            Err(e) => Some(format!("Failed to save config: {}", e)),
        }
    }
}
