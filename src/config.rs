use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub theme: Option<String>,
    pub style: Option<String>,
    pub number: Option<bool>,
    pub squeeze_blank: Option<bool>,
    pub show_ends: Option<bool>,
    pub show_tabs: Option<bool>,
}

impl Config {
    /// Mencari path default konfigurasi sesuai OS
    pub fn default_path() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return Some(PathBuf::from(appdata).join("rcat").join("config.toml"));
            }
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                return Some(PathBuf::from(userprofile).join(".config").join("rcat").join("config.toml"));
            }
        }

        #[cfg(not(windows))]
        {
            if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
                return Some(PathBuf::from(xdg).join("rcat").join("config.toml"));
            }
            if let Ok(home) = std::env::var("HOME") {
                return Some(PathBuf::from(home).join(".config").join("rcat").join("config.toml"));
            }
        }

        None
    }

    /// Membaca file konfigurasi dari path tertentu
    pub fn load_from_path(path: &Path) -> Option<Self> {
        if path.exists()
            && let Ok(content) = fs::read_to_string(path)
            && let Ok(cfg) = toml::from_str::<Config>(&content)
        {
            return Some(cfg);
        }
        None
    }

    /// Memuat konfigurasi dengan mempertimbangkan flag --config dan --no-config
    pub fn load(custom_path: Option<&Path>, no_config: bool) -> Self {
        if no_config {
            return Self::default();
        }

        if let Some(path) = custom_path {
            return Self::load_from_path(path).unwrap_or_default();
        }

        if let Some(default_path) = Self::default_path() {
            return Self::load_from_path(&default_path).unwrap_or_default();
        }

        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_toml() {
        let toml_str = r#"
            theme = "Solarized (dark)"
            style = "full"
            number = true
            squeeze_blank = true
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.as_deref(), Some("Solarized (dark)"));
        assert_eq!(config.style.as_deref(), Some("full"));
        assert_eq!(config.number, Some(true));
        assert_eq!(config.squeeze_blank, Some(true));
        assert_eq!(config.show_ends, None);
    }
}

