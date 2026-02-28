use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub pool: Option<String>,
    pub version: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub theme: Option<String>,
    pub background: Option<HashMap<String, BackgroundConfig>>, // [background.HDMI-1]
    pub time_config: Option<HashMap<String, DayTimeConfig>>,   // [timeConfig.HDMI-1]
    pub weather: Option<HashMap<String, WeatherConfig>>, // [weather.HDMI-1] or [weather.*]  for all
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BackgroundConfig {
    pub image: Option<String>,
    pub fill_mode: FillMode,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DayTimeConfig {
    pub day: String,
    pub night: String,
    pub day_range: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WeatherConfig {
    pub lat: f64,
    pub lon: f64,
    pub weather: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FillMode {
    Fill,
    Crop,
    Scale,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WeatherStates {
    Cloudy,
    Sunny,
    Raining,
    Snowing,
    Lighting,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WeatherImagesConf {
    pub image: String,
    pub weather: WeatherStates,
}

impl Config {
    pub fn load(config_file: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(config_file)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let config: Config = toml::from_slice(&data)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    /// Merge settings from a theme manifest into this config.
    /// Only fills in fields that are currently None.
    pub fn merge_theme(&mut self, theme_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let manifest_path = theme_path.join("manifest.toml");
        if !manifest_path.exists() {
            return Ok(());
        }

        let theme_config = Config::load(manifest_path)?;

        // Priority: Theme Manifest > User Config for trigger logic
        if theme_config.background.is_some() {
            self.background = theme_config.background;
        }
        if theme_config.time_config.is_some() {
            self.time_config = theme_config.time_config;
        }
        if theme_config.weather.is_some() {
            self.weather = theme_config.weather;
        }

        if self.name.is_none() || self.name == Some("wallman".to_string()) {
            self.name = theme_config.name;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pool: None,
            version: Some(1),
            name: Some("wallman".to_string()),
            description: Some("Dynamic wallpaper manager for Sway".to_string()),
            theme: None,
            background: None,
            time_config: None,
            weather: None,
        }
    }
}
