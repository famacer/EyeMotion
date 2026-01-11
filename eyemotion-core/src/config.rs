use crate::{AudioParams, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub audio: AudioParams,
    pub language: String,
    pub last_played_stage: i32,
}

impl Default for UserConfig {
    fn default() -> Self {
        UserConfig {
            audio: AudioParams::default(),
            language: "en".to_string(),
            last_played_stage: 1,
        }
    }
}

impl UserConfig {
    fn get_config_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let mut path = PathBuf::from(appdata);
                path.push("eyemotion");
                return Ok(path);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let mut path = PathBuf::from(home);
                path.push("Library/Application Support/eyemotion");
                return Ok(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let mut path = PathBuf::from(home);
                path.push(".config/eyemotion");
                return Ok(path);
            }
        }

        #[cfg(any(target_os = "ios", target_os = "android"))]
        {
            return Err(CoreError::Config(
                "Mobile path requires Tauri context".into(),
            ));
        }

        Ok(PathBuf::from("."))
    }

    pub fn load() -> Self {
        if let Ok(config_dir) = Self::get_config_path() {
            let config_file = config_dir.join("config.json");
            if config_file.exists() {
                if let Ok(content) = fs::read_to_string(&config_file) {
                    if let Ok(config) = serde_json::from_str::<UserConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        UserConfig::default()
    }

    pub fn save(&self) -> Result<()> {
        if let Ok(config_dir) = Self::get_config_path() {
            fs::create_dir_all(&config_dir)?;
            let config_file = config_dir.join("config.json");
            let json_string = serde_json::to_string_pretty(self)?;
            fs::write(config_file, json_string)?;
        }
        Ok(())
    }

    pub fn update_last_stage(&mut self, stage: i32) {
        if stage > self.last_played_stage {
            self.last_played_stage = stage;
        }
    }
}
