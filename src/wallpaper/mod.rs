pub mod apply;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

lazy_static! {
    /// Tracks active swaybg processes per output name.
    static ref PROCESS_TRACKER: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Kill the existing swaybg process for a specific output if it exists.
pub fn kill_for_output(output_name: &str) {
    let mut tracker = PROCESS_TRACKER.lock().unwrap();
    if let Some(mut child) = tracker.remove(output_name) {
        tracing::debug!("Killing existing swaybg for output '{}'", output_name);
        let _ = child.kill();
        let _ = child.wait(); // Prevent zombies
    }
}

/// Kill all tracked swaybg processes.
pub fn kill_all() {
    let mut tracker = PROCESS_TRACKER.lock().unwrap();
    tracing::debug!(
        "Killing all tracked swaybg processes (count: {})",
        tracker.len()
    );
    for (_, mut child) in tracker.drain() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// Register a new swaybg process for an output.
pub fn register_process(output_name: String, child: Child) {
    let mut tracker = PROCESS_TRACKER.lock().unwrap();
    tracker.insert(output_name, child);
}
