use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn default_ui_font_size() -> u8 {
    14
}

/// Application configuration, persisted to ~/.misakax/config.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// UI language (en, zh_CN)
    pub language: String,
    /// Theme: "light", "dark", "dim", "system"
    pub theme: String,
    /// Accent color (hex)
    pub accent_color: String,
    /// Reduce translucent surfaces and backdrop blur for readability
    #[serde(default)]
    pub reduced_transparency: bool,
    /// Base UI font size in pixels (e.g. 14)
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: u8,
    /// Default LLM model
    pub default_model: String,
    /// Log level
    pub log_level: String,
    /// Sidecar port
    pub sidecar_port: u16,
    /// Whether to auto-start sidecar
    pub auto_start_sidecar: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "system".to_string(),
            accent_color: "#6366f1".to_string(),
            reduced_transparency: false,
            ui_font_size: 14,
            default_model: "claude-sonnet-4-20250514".to_string(),
            log_level: "info".to_string(),
            sidecar_port: 9527,
            auto_start_sidecar: true,
        }
    }
}

/// Get the root config directory (~/.misakax/)
pub fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    Ok(home.join(".misakax"))
}

/// Get the database file path (~/.misakax/data/misaka.db)
pub fn db_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("data").join("misaka.db"))
}

/// Get the skills directory path (~/.misakax/skills/)
pub fn skills_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("skills"))
}

/// Get the managed skills directory path (~/.misakax/managed/skills/)
pub fn managed_skills_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("managed").join("skills"))
}

/// Get the logs directory path (~/.misakax/logs/)
pub fn logs_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("logs"))
}

/// Get the config file path (~/.misakax/config.yaml)
pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.yaml"))
}

/// Ensure all required directories exist.
pub fn ensure_directories() -> Result<()> {
    let dirs = [
        config_dir()?,
        config_dir()?.join("data"),
        skills_dir()?,
        managed_skills_dir()?,
        config_dir()?.join("plugins"),
        config_dir()?.join("models"),
        logs_dir()?,
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    tracing::info!(
        "Config directories initialized at: {}",
        config_dir()?.display()
    );
    Ok(())
}

/// Load config from YAML file, or create default if not exists.
pub fn load_config() -> Result<AppConfig> {
    let path = config_file_path()?;

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig =
            serde_yaml::from_str(&content).context("Failed to parse config.yaml")?;
        Ok(config)
    } else {
        let config = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

/// Save config to YAML file.
pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_file_path()?;
    let content = serde_yaml::to_string(config).context("Failed to serialize config")?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "system");
        assert_eq!(config.sidecar_port, 9527);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = AppConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let loaded: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.theme, loaded.theme);
        assert_eq!(config.language, loaded.language);
    }
}
