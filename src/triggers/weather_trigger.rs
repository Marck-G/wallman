use crate::{
    outputs::OutputResolver,
    trigger::{OutputChange, Trigger, TriggerResult},
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    collections::HashMap,
    result::Result as StdResult,
    time::{Duration, Instant},
};

/// Weather states that can trigger wallpaper changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeatherState {
    Clear,
    Cloudy,
    Rainy,
    Snowy,
    Stormy,
}

impl WeatherState {
    fn config_key(&self) -> &'static str {
        match self {
            Self::Clear => "sunny",
            Self::Cloudy => "cloudy",
            Self::Rainy => "raining",
            Self::Snowy => "snowing",
            Self::Stormy => "lighting",
        }
    }

    fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Clear,
            1 | 2 | 3 | 51 | 53 | 55 | 56 | 57 => Self::Cloudy,
            61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 => Self::Rainy,
            71 | 73 | 75 | 77 | 85 | 86 => Self::Snowy,
            95 | 96 | 99 => Self::Stormy,
            _ => Self::Cloudy,
        }
    }
}

/// Weather trigger that switches wallpapers based on current weather conditions.
///
/// Per-output state is tracked so each monitor can independently detect changes
/// (even though the weather source is currently global per lat/lon).
pub struct WeatherTrigger {
    /// Last known weather per output name.
    last_weather: HashMap<String, WeatherState>,
    client: Client,
    last_api_call: Option<Instant>,
    /// Cached weather result between API calls.
    cached_weather: Option<WeatherState>,
}

impl WeatherTrigger {
    pub fn new() -> Self {
        Self {
            last_weather: HashMap::new(),
            client: Client::new(),
            last_api_call: None,
            cached_weather: None,
        }
    }

    /// Fetch current weather from Open-Meteo using the lat/lon from the wildcard
    /// (or first available) weather config entry.
    fn fetch_weather(&mut self) -> StdResult<WeatherState, Box<dyn std::error::Error>> {
        // Rate-limit: at most once per 10 minutes.
        let now = Instant::now();
        if let Some(last) = self.last_api_call {
            if now.duration_since(last) < Duration::from_secs(600) {
                // Return cached value.
                if let Some(cached) = &self.cached_weather {
                    return Ok(cached.clone());
                }
            }
        }

        // Read config for coordinates.
        let state = crate::APP_STATE.get().unwrap().lock().unwrap();
        let config = state.config.clone();
        drop(state);

        let weather_map = match config.weather.as_ref() {
            Some(m) => m,
            None => return Err("No [weather.*] configuration found".into()),
        };

        // Use wildcard config for coordinates (weather is global, not per-output).
        let weather_cfg = weather_map
            .get("*")
            .or_else(|| weather_map.values().next())
            .ok_or_else(|| "Could not find any weather configuration entry")?;

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
            weather_cfg.lat, weather_cfg.lon
        );

        tracing::debug!("WeatherTrigger: fetching {}", url);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()?
            .error_for_status()?;

        let data: WeatherApiResponse = response.json()?;
        let state = WeatherState::from_code(data.current_weather.weathercode);

        tracing::info!("WeatherTrigger: current weather = {:?}", state);

        self.last_api_call = Some(now);
        self.cached_weather = Some(state.clone());
        Ok(state)
    }
}

impl Trigger for WeatherTrigger {
    fn init(&mut self) -> StdResult<(), Box<dyn std::error::Error>> {
        let state = crate::APP_STATE.get().unwrap().lock().unwrap();
        let config = state.config.clone();
        drop(state);

        if let Some(weather_map) = config.weather.as_ref() {
            let first = weather_map.get("*").or_else(|| weather_map.values().next());
            if let Some(wc) = first {
                tracing::info!(
                    "WeatherTrigger initializing: coordinates ({}, {})",
                    wc.lat,
                    wc.lon
                );
                // Perform first fetch during init to ensure state is ready (§Phase 2).
                if let Err(e) = self.fetch_weather() {
                    tracing::warn!("WeatherTrigger: initial fetch failed: {}", e);
                }
            }
        } else {
            tracing::warn!("WeatherTrigger: no [weather.*] configuration found");
        }

        Ok(())
    }

    fn evaluate(&mut self) -> StdResult<Option<TriggerResult>, Box<dyn std::error::Error>> {
        // ── 1. Clone config ───────────────────────────────────────────────
        let state = crate::APP_STATE.get().unwrap().lock().unwrap();
        let config = state.config.clone();

        let weather_map = match config.weather.as_ref() {
            Some(m) => m,
            None => {
                drop(state);
                return Ok(None);
            }
        };

        // ── 2. Fetch weather (rate-limited) ───────────────────────────────
        let current_weather = match self.fetch_weather() {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!("WeatherTrigger: could not fetch weather: {}", e);
                drop(state);
                return Ok(None);
            }
        };

        // ── 3. Detect outputs ─────────────────────────────────────────────
        let resolver = OutputResolver::detect()?;

        if resolver.outputs().is_empty() {
            drop(state);
            return Ok(None);
        }

        // ── 4. Resolve per-output weather config ─────────────────────────
        let resolved_weather = resolver.resolve_map(weather_map);

        // ── 5. Produce changes for outputs where weather flipped ──────────
        let mut changes: Vec<OutputChange> = Vec::new();

        for (output, wc) in &resolved_weather {
            // Check if the state actually changed for this output.
            if self.last_weather.get(output) == Some(&current_weather) {
                continue;
            }

            // Look up the image for the current weather state.
            let key = current_weather.config_key();
            let mut image_path = wc.weather.get(key).cloned();

            // Fallbacks for common variations/typos
            if image_path.is_none() {
                image_path = match current_weather {
                    WeatherState::Clear => wc.weather.get("clear").cloned(),
                    WeatherState::Rainy => wc.weather.get("rainy").cloned(),
                    WeatherState::Stormy => {
                        wc.weather
                            .get("stormy")
                            .or_else(|| wc.weather.get("ligthing")) // User typo fallback
                            .cloned()
                    }
                    _ => None,
                };
            }

            let image_path = match image_path {
                Some(p) => p,
                None => {
                    tracing::warn!(
                        "WeatherTrigger: no image for weather='{}' (or fallbacks) on output '{}' — skipping",
                        key,
                        output
                    );
                    continue;
                }
            };

            let resolved_path = state.resolve_image_path(&image_path);
            tracing::info!(
                "WeatherTrigger: output '{}' → {:?} → '{}'",
                output,
                current_weather,
                resolved_path
            );

            self.last_weather
                .insert(output.clone(), current_weather.clone());
            changes.push(OutputChange {
                output: output.clone(),
                image_path: resolved_path,
            });
        }

        drop(state);
        if changes.is_empty() {
            return Ok(None);
        }

        Ok(Some(TriggerResult { changes }))
    }

    fn interval(&self) -> u64 {
        // Check every 15 minutes to stay well within API rate limits.
        36000
    }
}

// ── Open-Meteo API response types ────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct WeatherApiResponse {
    current_weather: CurrentWeather,
}

#[derive(Deserialize, Default)]
struct CurrentWeather {
    weathercode: i32,
    #[allow(dead_code)]
    temperature: f64,
    #[allow(dead_code)]
    windspeed: f64,
    #[allow(dead_code)]
    winddirection: i32,
    #[allow(dead_code)]
    time: String,
}
