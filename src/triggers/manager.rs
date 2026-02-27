use std::time::{Duration, Instant};

use crate::trigger::Trigger;

/// Wrapper that tracks when a trigger should run next
pub struct ScheduledTrigger {
    pub trigger: Box<dyn Trigger>,
    pub next_run: Instant,
}

/// Manages all triggers and their execution
pub struct TriggerManager {
    triggers: Vec<ScheduledTrigger>,
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
        }
    }

    pub fn add(&mut self, trigger: Box<dyn Trigger>) {
        // Set next_run to now so it fires immediately upon start.
        let next_run = Instant::now();
        self.triggers.push(ScheduledTrigger { trigger, next_run });
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Trigger manager started");

        // Initialize all triggers
        for scheduled in &mut self.triggers {
            match scheduled.trigger.init() {
                Ok(()) => tracing::info!(
                    "Trigger initialized: {:?}",
                    std::any::type_name_of_val(&*scheduled.trigger)
                ),
                Err(e) => tracing::error!("Failed to initialize trigger: {}", e),
            }
        }

        loop {
            let now = Instant::now();

            for scheduled in self.triggers.iter_mut() {
                if now >= scheduled.next_run {
                    match scheduled.trigger.evaluate() {
                        Ok(Some(result)) => {
                            // Apply wallpaper change
                            if let Err(e) = crate::wallpaper::apply::apply(result) {
                                tracing::error!("Failed to apply wallpaper: {}", e);
                            }
                        }
                        Ok(None) => {
                            // No change needed
                            tracing::debug!("Trigger evaluated, no change needed");
                        }
                        Err(e) => {
                            tracing::error!("Trigger evaluation failed: {}", e);
                        }
                    }

                    // Schedule next run
                    scheduled.next_run = now + Duration::from_secs(scheduled.trigger.interval());
                }
            }

            // Sleep to prevent busy waiting
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}
