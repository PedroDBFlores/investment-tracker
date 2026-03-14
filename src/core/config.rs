use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub data_directory: Option<String>,
    pub default_currency: Option<String>,
    pub date_format: Option<String>,
    pub show_dividends: Option<bool>,
    pub color_output: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_directory: None,
            default_currency: Some("USD".to_string()),
            date_format: Some("YYYY-MM-DD".to_string()),
            show_dividends: Some(true),
            color_output: Some(true),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Result<Self, anyhow::Error> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("investment_tracker");

        let config_path = config_dir.join("config.json");

        if config_path.exists() {
            let config_file = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&config_file)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("investment_tracker");

        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(self)?;

        std::fs::write(config_path, config_json)?;

        Ok(())
    }

    pub fn get_data_directory(&self) -> Option<PathBuf> {
        self.data_directory.as_ref().map(|s| PathBuf::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::new();

        assert_eq!(config.default_currency, Some("USD".to_string()));
        assert_eq!(config.date_format, Some("YYYY-MM-DD".to_string()));
        assert_eq!(config.show_dividends, Some(true));
        assert_eq!(config.data_directory, None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            data_directory: Some("/custom/path".to_string()),
            default_currency: Some("EUR".to_string()),
            date_format: Some("DD/MM/YYYY".to_string()),
            show_dividends: Some(false),
            color_output: Some(false),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.data_directory, deserialized.data_directory);
        assert_eq!(config.default_currency, deserialized.default_currency);
        assert_eq!(config.date_format, deserialized.date_format);
        assert_eq!(config.show_dividends, deserialized.show_dividends);
        assert_eq!(config.color_output, deserialized.color_output);
    }
}
