use crate::{
    outputs::OutputResolver,
    trigger::{OutputChange, Trigger, TriggerResult},
};
use std::result::Result as StdResult;

/// Applies configured per-output wallpapers once at startup.
///
/// Reads `config.background`, resolves wildcard `"*"` entries against all
/// detected outputs, and emits a batch `TriggerResult` covering every output.
/// After the first successful evaluation it becomes a no-op.
pub struct StaticTrigger {
    executed: bool,
}

impl StaticTrigger {
    pub fn new() -> Self {
        Self { executed: false }
    }
}

impl Trigger for StaticTrigger {
    fn init(&mut self) -> StdResult<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn evaluate(&mut self) -> StdResult<Option<TriggerResult>, Box<dyn std::error::Error>> {
        if self.executed {
            return Ok(None);
        }

        // ── 1. Clone config ───────────────────────────────────────────────
        let state = crate::APP_STATE.get().unwrap().lock().unwrap();
        let config = state.config.clone();

        let background_map = match config.background.as_ref() {
            Some(m) => m,
            None => {
                tracing::warn!("StaticTrigger: no [background.*] configuration found");
                return Ok(None);
            }
        };

        // ── 2. Detect outputs ─────────────────────────────────────────────
        let resolver = OutputResolver::detect()?;

        if resolver.outputs().is_empty() {
            tracing::warn!("StaticTrigger: no active outputs detected — skipping");
            return Ok(None);
        }

        // ── 3. Resolve wildcard map ───────────────────────────────────────
        let resolved = resolver.resolve_map(background_map);

        // ── 4. Produce OutputChange per output ───────────────────────────
        let mut changes: Vec<OutputChange> = Vec::new();

        for (output, bg_cfg) in &resolved {
            if let Some(image_path) = &bg_cfg.image {
                let resolved_path = state.resolve_image_path(image_path);
                tracing::info!("StaticTrigger: output '{}' → '{}'", output, resolved_path);
                changes.push(OutputChange {
                    output: output.clone(),
                    image_path: resolved_path,
                });
            } else {
                tracing::warn!(
                    "StaticTrigger: output '{}' has a background config but no image — skipping",
                    output
                );
            }
        }

        if changes.is_empty() {
            return Ok(None);
        }

        self.executed = true;
        drop(state);
        Ok(Some(TriggerResult { changes }))
    }

    fn interval(&self) -> u64 {
        // Static trigger only fires once; keep a long interval so the manager
        // does not busy-spin the evaluate() call.
        60
    }
}
