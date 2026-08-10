use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use vnikey_core::engine::InputMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_method: String,
    pub toggle_modifier: String,
    pub toggle_key: String,
    pub start_enabled: bool,
    #[serde(default = "default_spell_check")]
    pub spell_check: bool,
}

fn default_spell_check() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_method: "telex".to_string(),
            toggle_modifier: "Control".to_string(),
            toggle_key: "Space".to_string(),
            start_enabled: true,
            spell_check: true,
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
        Self::load_from_path(&config_file)
    }

    pub fn load_from_path(path: &std::path::Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to parse config file at {:?}: {}. Using defaults.",
                            path, e
                        );
                        Config::default()
                    }
                },
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read config file at {:?}: {}. Using defaults.",
                        path, e
                    );
                    Config::default()
                }
            }
        } else {
            // Create default file
            if let Some(parent) = path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!(
                        "Warning: Failed to create config directory at {:?}: {}. Using defaults.",
                        parent, e
                    );
                    return Config::default();
                }
            }

            let default_config = Config::default();
            match toml::to_string(&default_config) {
                Ok(toml_string) => {
                    if let Err(e) = fs::write(path, toml_string) {
                        eprintln!(
                            "Warning: Failed to write default config to {:?}: {}",
                            path, e
                        );
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

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proj_dirs = ProjectDirs::from("", "", "vnikey")
            .ok_or("Could not determine configuration directory")?;
        let config_dir = proj_dirs.config_dir();

        let config_file = config_dir.join("config.toml");
        self.save_to_path(&config_file)
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let toml_string = toml::to_string(self)?;
        fs::write(path, toml_string)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_save_and_load() {
        let mut temp_dir = env::temp_dir();
        temp_dir.push(format!("vnikey_test_config_{}", std::process::id()));
        let config_path = temp_dir.join("config.toml");

        // Clean up before test just in case
        let _ = fs::remove_dir_all(&temp_dir);

        // 1. Create a modified config
        let mut config = Config::default();
        config.input_method = "vni".to_string();
        config.spell_check = false; // Testing the new field

        // 2. Save it
        config
            .save_to_path(&config_path)
            .expect("Failed to save config to path");

        // 3. Load it back
        let loaded_config = Config::load_from_path(&config_path);

        // 4. Verify fields
        assert_eq!(loaded_config.input_method, "vni");
        assert_eq!(loaded_config.spell_check, false);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
