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
    pub lat: Option<f64>,                                // Main config latitude
    pub lon: Option<f64>,                                // Main config longitude
    pub day_range: Option<String>,                       // Main config day range
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
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WeatherConfig {
    pub weather: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FillMode {
    /// Scale the image to cover the entire output, cropping if necessary (default).
    #[default]
    Fill,
    /// Same as fill — alias kept for backwards compatibility.
    Crop,
    /// Scale the image to fit within the output, preserving aspect ratio (may letterbox).
    Fit,
    /// Scale the image to exactly match the output dimensions, ignoring aspect ratio.
    Scale,
    /// Tile the image at its original size across the output.
    Tile,
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
    /// Only fills in fields that are currently None, except for lat, lon, and day_range
    /// which are preserved from the user config.
    pub fn merge_theme(&mut self, theme_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let manifest_path = theme_path.join("manifest.toml");
        if !manifest_path.exists() {
            return Ok(());
        }

        let theme_config = Config::load(manifest_path)?;

        // Preserve user's lat, lon, and day_range settings
        let user_lat = self.lat.clone();
        let user_lon = self.lon.clone();
        let user_day_range = self.day_range.clone();

        // Priority: Theme Manifest > User Config for trigger logic
        // But preserve user's main config settings for lat, lon, day_range
        if theme_config.background.is_some() {
            self.background = theme_config.background;
        }
        if theme_config.time_config.is_some() {
            self.time_config = theme_config.time_config;
        }
        if theme_config.weather.is_some() {
            self.weather = theme_config.weather;
        }

        // Preserve user's main config fields
        self.lat = user_lat;
        self.lon = user_lon;
        self.day_range = user_day_range;

        // Update name and description from theme if not set or default
        // Note: We don't preserve user's name/description/theme as these should come from the theme
        if self.name.is_none() || self.name == Some("wallman".to_string()) {
            self.name = theme_config.name;
        }
        if self.description.is_none() {
            self.description = theme_config.description;
        }
        if self.theme.is_none() {
            self.theme = theme_config.theme;
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
            lat: None,
            lon: None,
            day_range: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_merge_theme_preserves_user_settings() {
        // Create a user config with lat, lon, and day_range set
        let mut user_config = Config::default();
        user_config.lat = Some(40.7128); // New York
        user_config.lon = Some(-74.0060);
        user_config.day_range = Some("06-18".to_string()); // 6 AM to 6 PM
        user_config.pool = Some("/old/theme/path".to_string());
        user_config.name = Some("My Custom Config".to_string());
        user_config.description = Some("User's custom configuration".to_string());

        // Create a temporary theme directory and manifest
        let temp_dir = std::env::temp_dir().join("wallman_test_theme");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
        fs::create_dir_all(&temp_dir).unwrap();

        let theme_config = Config {
            pool: Some("/theme/pool/path".to_string()),
            version: Some(1),
            name: Some("Test Theme".to_string()),
            description: Some("A test theme".to_string()),
            theme: Some("test-theme".to_string()),
            background: Some(std::collections::HashMap::from([(
                "HDMI-1".to_string(),
                BackgroundConfig {
                    image: Some("theme-background.jpg".to_string()),
                    fill_mode: FillMode::Fill,
                },
            )])),
            time_config: Some(std::collections::HashMap::from([(
                "HDMI-1".to_string(),
                DayTimeConfig {
                    day: "day-image.jpg".to_string(),
                    night: "night-image.jpg".to_string(),
                },
            )])),
            weather: Some(std::collections::HashMap::from([(
                "*".to_string(),
                WeatherConfig {
                    weather: std::collections::HashMap::from([
                        ("sunny".to_string(), "sunny.jpg".to_string()),
                        ("cloudy".to_string(), "cloudy.jpg".to_string()),
                    ]),
                },
            )])),
            lat: Some(51.5074), // London (different from user)
            lon: Some(-0.1278),
            day_range: Some("07-19".to_string()), // Different from user
        };

        let manifest_path = temp_dir.join("manifest.toml");
        theme_config.save_to_file(&manifest_path).unwrap();

        // Test the merge behavior
        let mut merged_config = user_config.clone();
        merged_config.merge_theme(temp_dir.clone()).unwrap();

        // Verify that user's lat, lon, and day_range are preserved
        assert_eq!(merged_config.lat, Some(40.7128));
        assert_eq!(merged_config.lon, Some(-74.0060));
        assert_eq!(merged_config.day_range, Some("06-18".to_string()));

        // Verify that theme's other settings are applied
        // Note: name should be preserved if it's not the default "wallman"
        assert_eq!(merged_config.name, Some("My Custom Config".to_string()));
        assert_eq!(merged_config.description, Some("A test theme".to_string()));
        assert_eq!(merged_config.theme, Some("test-theme".to_string()));
        assert!(merged_config.background.is_some());
        assert!(merged_config.time_config.is_some());
        assert!(merged_config.weather.is_some());

        // Verify that pool is updated from theme
        assert_eq!(merged_config.pool, Some("/theme/pool/path".to_string()));

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
