use crate::{
    config::DayTimeConfig,
    outputs::OutputResolver,
    trigger::{OutputChange, Trigger, TriggerResult},
};
use chrono::{Local, Timelike};
use std::{collections::HashMap, result::Result as StdResult};
use tracing::info;

/// Day/Night trigger — switches wallpapers based on the time of day.
///
/// Internal state tracks the last day/night flag *per output* so a change on
/// one monitor does not suppress an update for another.
pub struct DayTimeTrigger {
    /// Keyed by output name. `true` = currently showing day wallpaper.
    last_state: HashMap<String, bool>,
}

impl DayTimeTrigger {
    pub fn new() -> Self {
        Self {
            last_state: HashMap::new(),
        }
    }

    /// Determine whether it is currently daytime for a given output's time config.
    fn is_daytime_for(&self, time_cfg: &DayTimeConfig) -> bool {
        let hour = Local::now().hour();
        let day_range = match time_cfg.day_range.as_ref() {
            Some(range) => range.clone(),
            None => {
                format!(
                    "{}-{}",
                    crate::constants::day_start(),
                    crate::constants::day_end()
                )
            }
        };
        let day_start = day_range.split('-').next().unwrap().parse::<u32>().unwrap();
        let night_start = day_range
            .split('-')
            .next_back()
            .unwrap()
            .parse::<u32>()
            .unwrap();
        tracing::debug!(
            "DayTimeTrigger: day_range={} day_start={} night_start={}",
            day_range,
            day_start,
            night_start
        );
        if day_start < night_start {
            // Normal case: daytime window e.g. 06:00 – 18:00
            hour >= day_start && hour < night_start
        } else {
            // Overnight case: daytime window wraps midnight e.g. 22:00 – 08:00
            hour >= day_start || hour < night_start
        }
    }
}

impl Trigger for DayTimeTrigger {
    fn init(&mut self) -> StdResult<(), Box<dyn std::error::Error>> {
        // ── 1. Clone config ───────────────────────────────────────────────
        let state = crate::APP_STATE.get().unwrap().lock().unwrap();
        let config = state.config.clone();
        drop(state);

        let time_map = match config.time_config.as_ref() {
            Some(m) => m,
            None => {
                tracing::info!("DayTimeTrigger: no [timeConfig.*] configuration — init skipped");
                return Ok(());
            }
        };

        // Detect outputs to log status
        let resolver = OutputResolver::detect()?;
        let resolved_time = resolver.resolve_map(time_map);

        for (output, time_cfg) in &resolved_time {
            let is_day = self.is_daytime_for(time_cfg);
            tracing::info!(
                "DayTimeTrigger ready: output '{}' (current={})",
                output,
                if is_day { "day" } else { "night" }
            );
        }

        Ok(())
    }

    fn evaluate(&mut self) -> StdResult<Option<TriggerResult>, Box<dyn std::error::Error>> {
        info!("DayTimeTrigger evaluate started");
        // ── 1. Clone config ───────────────────────────────────────────────
        let state = crate::APP_STATE.get().unwrap().lock().unwrap();
        let config = state.config.clone();

        let time_map = match config.time_config.as_ref() {
            Some(m) => m,
            None => {
                drop(state);
                return Ok(None);
            }
        };

        // ── 2. Detect outputs ─────────────────────────────────────────────
        let resolver = OutputResolver::detect()?;
        info!("DayTimeTrigger resolver detected outputs");

        let resolved_time = resolver.resolve_map(time_map);
        let resolved_bg = config
            .background
            .as_ref()
            .map(|m| resolver.resolve_map(m))
            .unwrap_or_default();

        info!("DayTimeTrigger resolved maps for all outputs");

        // ── 4. Determine changes per output ──────────────────────────────
        let mut changes: Vec<OutputChange> = Vec::new();
        info!("DayTimeTrigger determining changes per output");
        for (output, time_cfg) in &resolved_time {
            let is_day = self.is_daytime_for(time_cfg);

            // Only emit a change if the state actually flipped for this output.
            if self.last_state.get(output) == Some(&is_day) {
                continue;
            }

            // Pick the correct image for this output and time-of-day.
            let bg_cfg = resolved_bg.get(output);

            let image_path = if is_day {
                // day image = time_cfg.day field if it looks like a path,
                // otherwise fall back to the generic background image.
                if time_cfg.day.contains('/') || time_cfg.day.contains('.') {
                    time_cfg.day.clone()
                } else {
                    bg_cfg.and_then(|c| c.image.clone()).unwrap_or_else(|| {
                        tracing::warn!("No day image path found for output '{}'", output);
                        String::new()
                    })
                }
            } else {
                // night image = time_cfg.night field if it looks like a path.
                if time_cfg.night.contains('/') || time_cfg.night.contains('.') {
                    time_cfg.night.clone()
                } else {
                    bg_cfg.and_then(|c| c.image.clone()).unwrap_or_else(|| {
                        tracing::warn!("No night image path found for output '{}'", output);
                        String::new()
                    })
                }
            };

            if image_path.is_empty() {
                continue;
            }

            let resolved_path = state.resolve_image_path(&image_path);
            tracing::info!(
                "DayTimeTrigger: output '{}' → {} → '{}'",
                output,
                if is_day { "day" } else { "night" },
                resolved_path
            );

            self.last_state.insert(output.clone(), is_day);
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
        // Check every minute.
        60
    }
}
