use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone
, PartialEq, PartialOrd)]
pub struct Config {
    pub image: Option<String>,
    pub fill_mode: String,
    pub time_config: Option<DayTimeConfig>,
    pub wheather: Option<WeatherConfig>
}

#[derive(Serialize, Deserialize, Clone
, PartialEq, PartialOrd)]
pub struct DayTimeConfig{
    pub day: String,
    pub nigth: String,
    pub day_range: Option<String>,
}
#[derive(Serialize, Deserialize, Clone
, PartialEq, PartialOrd)]
pub struct WeatherConfig{
    pub lat: f64,
    pub lon: f64,
    pub images: Vec<WatherImagesConf>
}

#[derive(Serialize, Deserialize, Clone
, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum WeatherStates {
    CLOUDY,
    SUNNY,
    RAINING,
    SNOWING,
    LIGHTING,
}

#[derive(Serialize, Deserialize, Clone
, PartialEq, PartialOrd)]
pub struct WatherImagesConf{
    pub image: String,
    pub weather: WeatherStates
}