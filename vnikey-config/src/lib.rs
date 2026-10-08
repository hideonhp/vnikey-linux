use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use vnikey_core::engine::InputMethod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_method: String,
    pub toggle_modifier: String,
    pub toggle_key: String,
    #[serde(default)]
    pub cycle_method_modifier: String,
    #[serde(default)]
    pub cycle_method_key: String,
    pub start_enabled: bool,
    #[serde(default = "default_spell_check")]
    pub spell_check: bool,
    #[serde(default = "default_vim_mode")]
    pub vim_mode: bool,
    #[serde(default = "default_per_window_state")]
    pub per_window_state: bool,
    #[serde(default = "default_notification_enabled")]
    pub notification_enabled: bool,
    /// Milliseconds to wait after injecting text via the X11 clipboard before
    /// restoring the previous clipboard contents.  On slow machines a larger
    /// value (e.g. 50) prevents a race where the target app has not yet read
    /// the clipboard before it is overwritten.  Default: 20 ms.
    #[serde(default = "default_clipboard_timeout_ms")]
    pub clipboard_timeout_ms: u64,
    #[serde(skip)]
    pub macros: std::collections::HashMap<String, String>,
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

fn default_notification_enabled() -> bool {
    true
}

fn default_clipboard_timeout_ms() -> u64 {
    20
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_method: "telex".to_string(),
            toggle_modifier: "control".to_string(),
            toggle_key: "space".to_string(),
            cycle_method_modifier: "".to_string(),
            cycle_method_key: "".to_string(),
            start_enabled: true,
            spell_check: true,
            vim_mode: false,
            per_window_state: false,
            notification_enabled: true,
            clipboard_timeout_ms: default_clipboard_timeout_ms(),
            macros: std::collections::HashMap::new(),
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
                        // --- BẮT ĐẦU PHẦN MỚI (Load macros) ---
                        if let Some(parent) = path.parent() {
                            let abbr_path = parent.join("abbr.toml");
                            if let Ok(abbr_contents) = fs::read_to_string(&abbr_path) {
                                if let Ok(parsed) = toml::from_str(&abbr_contents) {
                                    config.macros = parsed;
                                }
                            } else {
                                let default_abbr =
                                    "# VNIKey Abbreviation (Gõ tắt)\n# vn = \"Việt Nam\"\n";
                                let _ = fs::write(&abbr_path, default_abbr);
                            }
                        }
                        // --- KẾT THÚC PHẦN MỚI ---

                        config.toggle_modifier = config.toggle_modifier.to_lowercase();
                        config.toggle_key = config.toggle_key.to_lowercase();
                        config.cycle_method_modifier = config.cycle_method_modifier.to_lowercase();
                        config.cycle_method_key = config.cycle_method_key.to_lowercase();
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
        } else if self.input_method.eq_ignore_ascii_case("viqr") {
            InputMethod::Viqr
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

    pub fn get_cycle_method_modifier_normalized(&self) -> &str {
        self.cycle_method_modifier.as_str()
    }

    pub fn get_cycle_method_key_normalized(&self) -> &str {
        self.cycle_method_key.as_str()
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

        // Patch-save: nếu file đã tồn tại, chỉ update từng key một thay vì
        // overwrite toàn bộ. Điều này bảo toàn comment và các field không
        // được khai báo trong Config struct (custom user fields).
        if path.exists() {
            let existing = fs::read_to_string(path).unwrap_or_default();
            let patched = patch_toml(
                &existing,
                &[
                    ("input_method", format!("\"{}\"", self.input_method)),
                    ("toggle_modifier", format!("\"{}\"", self.toggle_modifier)),
                    ("toggle_key", format!("\"{}\"", self.toggle_key)),
                    (
                        "cycle_method_modifier",
                        format!("\"{}\"", self.cycle_method_modifier),
                    ),
                    (
                        "cycle_method_key",
                        format!("\"{}\"", self.cycle_method_key),
                    ),
                    ("start_enabled", self.start_enabled.to_string()),
                    ("spell_check", self.spell_check.to_string()),
                    ("vim_mode", self.vim_mode.to_string()),
                    ("per_window_state", self.per_window_state.to_string()),
                    ("notification_enabled", self.notification_enabled.to_string()),
                    (
                        "clipboard_timeout_ms",
                        self.clipboard_timeout_ms.to_string(),
                    ),
                ],
            );
            fs::write(path, patched)?;
            return Ok(());
        }

        // File chưa tồn tại: tạo mới bằng toml::to_string
        let toml_string = toml::to_string(self)?;
        fs::write(path, toml_string)?;

        Ok(())
    }
}

/// Patch một TOML string bằng cách cập nhật từng `key = value` trên từng dòng.
/// Nếu key chưa tồn tại trong file, append xuống cuối.
/// Giữ nguyên comment (#...) và các key không trong danh sách patch.
fn patch_toml(content: &str, patches: &[(&str, String)]) -> String {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut patched_keys = std::collections::HashSet::new();

    for line in &mut lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue; // giữ nguyên comment
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            if let Some((_, new_val)) = patches.iter().find(|(k, _)| *k == key) {
                // Preserve leading whitespace
                let indent: String = line
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect();
                *line = format!("{indent}{key} = {new_val}");
                patched_keys.insert(key.to_string());
            }
        }
    }

    // Append các key chưa tồn tại trong file
    for (key, val) in patches {
        if !patched_keys.contains(*key) {
            lines.push(format!("{key} = {val}"));
        }
    }

    // Đảm bảo trailing newline
    let mut result = lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
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
        config.spell_check = false;
        config.clipboard_timeout_ms = 50;

        // 2. Save it
        config
            .save_to_path(&config_path)
            .expect("Failed to save config to path");

        // 3. Load it back
        let loaded_config = Config::load_from_path(&config_path);

        // 4. Verify fields
        assert_eq!(loaded_config.input_method, "vni");
        assert_eq!(loaded_config.spell_check, false);
        assert_eq!(loaded_config.clipboard_timeout_ms, 50);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_default_clipboard_timeout() {
        let config = Config::default();
        assert_eq!(config.clipboard_timeout_ms, 20);
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

        // Test VIQR variants
        config.input_method = "viqr".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Viqr);

        config.input_method = "VIQR".to_string();
        assert_eq!(config.get_input_method(), InputMethod::Viqr);

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

    #[test]
    fn test_xdg_config_path() {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "vnikey") {
            let config_dir = proj_dirs.config_dir();
            // Trên Linux, đường dẫn này thường kết thúc bằng "vnikey"
            assert!(config_dir.ends_with("vnikey"));
        }
    }

    #[test]
    fn test_patch_toml_preserves_comments() {
        let toml = "# My comment\ninput_method = \"telex\"\nspell_check = true\n";
        let result = patch_toml(toml, &[("input_method", "\"vni\"".to_string())]);
        assert!(result.contains("# My comment"), "Comment phải được giữ lại");
        assert!(result.contains("input_method = \"vni\""), "Key phải được update");
        assert!(result.contains("spell_check = true"), "Các key khác không bị xóa");
    }

    #[test]
    fn test_patch_toml_appends_new_key() {
        let toml = "input_method = \"telex\"\n";
        let result = patch_toml(toml, &[("vim_mode", "true".to_string())]);
        assert!(result.contains("vim_mode = true"), "Key mới phải được append");
        assert!(result.contains("input_method = \"telex\""), "Key cũ giữ nguyên");
    }

    #[test]
    fn test_patch_toml_preserves_unknown_fields() {
        // User thêm custom field không có trong Config struct
        let toml = "input_method = \"telex\"\ncustom_user_field = 42\n";
        let result = patch_toml(
            toml,
            &[("input_method", "\"vni\"".to_string())],
        );
        assert!(result.contains("custom_user_field = 42"), "Custom field phải được giữ lại");
        assert!(result.contains("input_method = \"vni\""));
    }

    #[test]
    fn test_save_to_path_preserves_comments_on_update() {
        let mut temp_dir = std::env::temp_dir();
        temp_dir.push(format!("vnikey_test_patch_{}", std::process::id()));
        let config_path = temp_dir.join("config.toml");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Viết file ban đầu có comment
        let initial = "# VNIKey config\ninput_method = \"telex\"\nspell_check = true\n# custom_note = keep me\n";
        fs::write(&config_path, initial).unwrap();

        // Save với input_method đổi sang vni
        let mut cfg = Config::default();
        cfg.input_method = "vni".to_string();
        cfg.save_to_path(&config_path).unwrap();

        let saved = fs::read_to_string(&config_path).unwrap();
        assert!(saved.contains("# VNIKey config"), "Comment đầu file phải được giữ");
        assert!(saved.contains("input_method = \"vni\""), "input_method phải được update");
        assert!(saved.contains("# custom_note = keep me"), "Inline comment phải được giữ");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
