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
    #[serde(default = "default_vim_mode")]
    pub vim_mode: bool,
    #[serde(default = "default_per_window_state")]
    pub per_window_state: bool,
}

fn default_spell_check() -> bool {
    true
}

fn default_vim_mode() -> bool {
    false
}

fn default_per_window_state() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_method: "telex".to_string(),
            toggle_modifier: "control".to_string(),
            toggle_key: "space".to_string(),
            start_enabled: true,
            spell_check: true,
            vim_mode: false,
            per_window_state: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(proj_dirs) = ProjectDirs::from("", "", "vnikey") else {
            eprintln!("Warning: Could not determine configuration directory. Using defaults.");
            return Config::default();
        };

        let config_file = proj_dirs.config_dir().join("config.toml");
        Self::load_from_path(&config_file)
    }

    pub fn load_from_path(path: &std::path::Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        config.toggle_modifier = config.toggle_modifier.to_lowercase();
                        config.toggle_key = config.toggle_key.to_lowercase();
                        config
                    }
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
            if let Some(parent) = path.parent()
                && let Err(e) = fs::create_dir_all(parent)
            {
                eprintln!(
                    "Warning: Failed to create config directory at {:?}: {}. Using defaults.",
                    parent, e
                );
                return Config::default();
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
        if self.input_method.eq_ignore_ascii_case("vni") {
            InputMethod::Vni
        } else {
            InputMethod::Telex // Default/fallback
        }
    }

    pub fn get_toggle_modifier_normalized(&self) -> &str {
        self.toggle_modifier.as_str()
    }

    pub fn get_toggle_key_normalized(&self) -> &str {
        self.toggle_key.as_str()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proj_dirs = ProjectDirs::from("", "", "vnikey")
            .ok_or("Could not determine configuration directory")?;
        let config_dir = proj_dirs.config_dir();

        let config_file = config_dir.join("config.toml");
        self.save_to_path(&config_file)
    }

    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
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

    #[test]
    fn test_get_input_method() {
        let mut config = Config::default();

        // Test VNI variants
        config.input_method = "vni".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Vni);

        config.input_method = "VNI".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Vni);

        config.input_method = "Vni".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Vni);

        // Test Telex (default)
        config.input_method = "telex".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Telex);

        config.input_method = "TELEX".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Telex);

        // Test fallback (invalid inputs default to Telex)
        config.input_method = "unknown".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Telex);

        config.input_method = "".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Telex);
    }

    #[test]
    fn test_get_toggle_modifier_normalized() {
        let mut config = Config::default();

        config.toggle_modifier = "control".to_string();
        assert_eq!(config.get_toggle_modifier_normalized(), "control");

        config.toggle_modifier = "".to_string();
        assert_eq!(config.get_toggle_modifier_normalized(), "");
    }

    #[test]
    fn test_get_toggle_key_normalized() {
        let mut config = Config::default();

        config.toggle_key = "space".to_string();
        assert_eq!(config.get_toggle_key_normalized(), "space");

        config.toggle_key = "".to_string();
        assert_eq!(config.get_toggle_key_normalized(), "");
    }
}
