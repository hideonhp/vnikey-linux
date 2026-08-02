use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use vnikey_core::engine::InputMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_method: String,
    pub toggle_modifier: String,
    pub toggle_key: String,
    pub start_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_method: "telex".to_string(),
            toggle_modifier: "Control".to_string(),
            toggle_key: "Space".to_string(),
            start_enabled: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let proj_dirs = ProjectDirs::from("", "", "vnikey");

        let config_dir = match proj_dirs {
            Some(dirs) => dirs.config_dir().to_path_buf(),
            None => {
                eprintln!("Warning: Could not determine configuration directory. Using defaults.");
                return Config::default();
            }
        };

        let config_file = config_dir.join("config.toml");

        if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("Warning: Failed to parse config file at {:?}: {}. Using defaults.", config_file, e);
                            Config::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read config file at {:?}: {}. Using defaults.", config_file, e);
                    Config::default()
                }
            }
        } else {
            // Create default file
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!("Warning: Failed to create config directory at {:?}: {}. Using defaults.", config_dir, e);
                return Config::default();
            }

            let default_config = Config::default();
            match toml::to_string(&default_config) {
                Ok(toml_string) => {
                    if let Err(e) = fs::write(&config_file, toml_string) {
                        eprintln!("Warning: Failed to write default config to {:?}: {}", config_file, e);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to serialize default config: {}", e);
                }
            }

            default_config
        }
    }

    pub fn get_input_method(&self) -> InputMethod {
        match self.input_method.to_lowercase().as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex, // Default/fallback
        }
    }

    pub fn get_toggle_modifier_normalized(&self) -> String {
        self.toggle_modifier.to_lowercase()
    }

    pub fn get_toggle_key_normalized(&self) -> String {
        self.toggle_key.to_lowercase()
    }
}
