use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub dll_path: Option<String>,
    pub last_process: Option<String>,
    pub auto_inject: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dll_path: None,
            last_process: None,
            auto_inject: false,
        }
    }
}

impl Config {
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rust_injector");
        path.push("config.json");
        path
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

    pub fn save(&self) {
        let config_path = Self::get_config_path();
        
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        if let Ok(config_str) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&config_path, config_str);
        }
    }
}
