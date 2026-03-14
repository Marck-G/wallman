use crate::{
    config::{DayTimeConfig, FillMode},
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

    #[allow(dead_code, unused_variables)]
    /// Determine whether it is currently daytime for a given output's time config.
    fn is_daytime_for(&self, time_cfg: &DayTimeConfig) -> bool {
        let hour = Local::now().hour();

        // Try to get day_range from main config first, then use default
        let day_range = {
            let state = crate::APP_STATE.get().unwrap().lock().unwrap();
            let config = state.config.clone();
            drop(state);

            match config.day_range.as_ref() {
                Some(range) => range.clone(),
                None => {
                    format!(
                        "{}-{}",
                        crate::constants::day_start(),
                        crate::constants::day_end()
                    )
                }
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
        drop(state);
        let fill_mode = config.fill_mode.clone().unwrap_or(FillMode::default());
        let time_map = match config.time_config.as_ref() {
            Some(m) => m,
            None => {
                return Ok(None);
            }
        };

        // ── 2. Detect outputs ─────────────────────────────────────────────
        let resolver = OutputResolver::detect()?;
        info!("DayTimeTrigger resolver detected outputs");

        let resolved_time = resolver.resolve_map(time_map);

        info!(
            "DayTimeTrigger resolved maps for all outputs: {:?}",
            resolved_time.keys().collect::<Vec<_>>()
        );

        // ── 4. Determine changes per output ──────────────────────────────
        let mut changes: Vec<OutputChange> = Vec::new();
        info!("DayTimeTrigger determining changes per output");

        if resolved_time.is_empty() {
            info!("DayTimeTrigger: no outputs with time config - cannot determine changes");
            return Ok(None);
        }
        for (output, time_cfg) in &resolved_time {
            let is_day = self.is_daytime_for(time_cfg);
            info!(
                "Processing output '{}': is_day={}, time_cfg.day='{}', time_cfg.night='{}'",
                output, is_day, time_cfg.day, time_cfg.night
            );

            // Only emit a change if the state actually flipped for this output.
            if self.last_state.get(output) == Some(&is_day) {
                info!(
                    "Output '{}': state unchanged (last_state={:?}), skipping",
                    output,
                    self.last_state.get(output)
                );
                continue;
            }

            info!("Output '{}': state changed, will apply wallpaper", output);

            // Pick the correct image for this output and time-of-day.
            // Fallback: try other outputs' time_config entries if current output has no direct path.
            let fallback_time_cfg = resolved_time
                .values()
                .find(|cfg| cfg != &time_cfg && (cfg.day.contains('/') || cfg.day.contains('.')));

            let image_source: &str;
            let image_path = if is_day {
                // day image = time_cfg.day field if it looks like a path,
                // otherwise fall back to another output's time_config.
                if time_cfg.day.contains('/') || time_cfg.day.contains('.') {
                    image_source = "time_config.day (direct path)";
                    time_cfg.day.clone()
                } else {
                    image_source = "time_config fallback (from other output)";
                    fallback_time_cfg.map(|c| c.day.clone()).unwrap_or_else(|| {
                        tracing::warn!("No day image path found for output '{}'", output);
                        String::new()
                    })
                }
            } else {
                // night image = time_cfg.night field if it looks like a path,
                // otherwise fall back to another output's time_config.
                if time_cfg.night.contains('/') || time_cfg.night.contains('.') {
                    image_source = "time_config.night (direct path)";
                    time_cfg.night.clone()
                } else {
                    image_source = "time_config fallback (from other output)";
                    fallback_time_cfg
                        .map(|c| c.night.clone())
                        .unwrap_or_else(|| {
                            tracing::warn!("No night image path found for output '{}'", output);
                            String::new()
                        })
                }
            };

            tracing::debug!(
                "DayTimeTrigger DEBUG: output '{}' using source '{}'",
                output,
                image_source
            );

            if image_path.is_empty() {
                tracing::warn!("No image path found for output '{}', skipping", output);
                continue;
            }
            let state = crate::APP_STATE.get().unwrap().lock().unwrap();

            let resolved_path = state.resolve_image_path(&image_path);
            tracing::info!(
                "DayTimeTrigger: output '{}' → {} → '{}'",
                output,
                if is_day { "day" } else { "night" },
                resolved_path
            );

            drop(state);
            self.last_state.insert(output.clone(), is_day);
            changes.push(OutputChange {
                output: output.clone(),
                image_path: resolved_path,
                fill_mode: fill_mode.clone(),
            });
        }

        if changes.is_empty() {
            info!("DayTimeTrigger: no changes (either no outputs or state already matches)");
            return Ok(None);
        }

        info!("DayTimeTrigger: {} changes to apply", changes.len());

        Ok(Some(TriggerResult { changes }))
    }

    fn interval(&self) -> u64 {
        // Check every minute.
        60
    }
}
