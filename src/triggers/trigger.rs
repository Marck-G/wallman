use std::result::Result as StdResult;

/// A single output → image assignment decided by a trigger.
#[derive(Debug, Clone)]
pub struct OutputChange {
    pub output: String,
    pub image_path: String,
}

/// Result of a trigger evaluation — carries decisions for one or more outputs.
///
/// Replaces the old single-output `TriggerResult`.
#[derive(Debug, Clone)]
pub struct TriggerResult {
    pub changes: Vec<OutputChange>,
}

impl TriggerResult {
    /// Convenience constructor for a single-output result.
    pub fn single(output: impl Into<String>, image_path: impl Into<String>) -> Self {
        Self {
            changes: vec![OutputChange {
                output: output.into(),
                image_path: image_path.into(),
            }],
        }
    }

    /// Returns true when there are no output changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Trait that all triggers must implement.
pub trait Trigger: Send {
    /// Called once when the trigger starts (before the first evaluate loop).
    fn init(&mut self) -> StdResult<(), Box<dyn std::error::Error>>;

    /// Called periodically by the manager.
    ///
    /// Returns `Some(TriggerResult)` whose `changes` may cover multiple outputs
    /// if a wallpaper change is needed, or `None` when nothing changed.
    fn evaluate(&mut self) -> StdResult<Option<TriggerResult>, Box<dyn std::error::Error>>;

    /// How often (in seconds) the manager should call `evaluate`.
    fn interval(&self) -> u64;
}
