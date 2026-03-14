pub mod apply;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A handle to a running wallpaper surface for one output.
/// Dropping this handle or calling `stop()` tears down the Wayland surface.
pub struct WallpaperHandle {
    /// Channel sender — send `()` to request shutdown of the event loop thread.
    stop_tx: std::sync::mpsc::SyncSender<()>,
    /// Thread handle.
    thread: Option<std::thread::JoinHandle<()>>,
}

impl WallpaperHandle {
    /// Signal the event loop to exit and wait for the thread to finish.
    pub fn stop(mut self) {
        let _ = self.stop_tx.send(());
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

lazy_static! {
    /// Tracks active wallpaper handles per output name.
    static ref SURFACE_TRACKER: Arc<Mutex<HashMap<String, WallpaperHandle>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Tear down the existing wallpaper surface for a specific output, if any.
pub fn kill_for_output(output_name: &str) {
    let mut tracker = SURFACE_TRACKER.lock().unwrap();
    if let Some(handle) = tracker.remove(output_name) {
        tracing::debug!("Stopping existing wallpaper surface for output '{}'", output_name);
        handle.stop();
    }
}

/// Tear down all wallpaper surfaces.
pub fn kill_all() {
    let mut tracker = SURFACE_TRACKER.lock().unwrap();
    tracing::debug!(
        "Stopping all wallpaper surfaces (count: {})",
        tracker.len()
    );
    for (_, handle) in tracker.drain() {
        handle.stop();
    }
}

/// Register a new wallpaper handle for an output, replacing any existing one.
pub fn register_handle(output_name: String, handle: WallpaperHandle) {
    let mut tracker = SURFACE_TRACKER.lock().unwrap();
    // If there's already one, stop it first.
    if let Some(old) = tracker.remove(&output_name) {
        old.stop();
    }
    tracker.insert(output_name, handle);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Create a no-op WallpaperHandle backed by a thread that waits for the
    /// stop signal, and an Arc<bool> that flips to true when the thread exits.
    fn dummy_handle() -> (WallpaperHandle, Arc<Mutex<bool>>) {
        let exited = Arc::new(Mutex::new(false));
        let exited_clone = Arc::clone(&exited);

        let (stop_tx, stop_rx) = std::sync::mpsc::sync_channel::<()>(1);
        let thread = std::thread::spawn(move || {
            // Block until stop is requested.
            let _ = stop_rx.recv();
            *exited_clone.lock().unwrap() = true;
        });

        let handle = WallpaperHandle {
            stop_tx,
            thread: Some(thread),
        };

        (handle, exited)
    }

    #[test]
    fn handle_stop_joins_thread() {
        let (handle, exited) = dummy_handle();
        assert!(!*exited.lock().unwrap(), "thread should still be running");
        handle.stop();
        assert!(*exited.lock().unwrap(), "thread should have exited after stop()");
    }

    #[test]
    fn register_replaces_existing_handle() {
        // Use a unique output name to avoid colliding with other tests.
        let output = "__test_register_replace__".to_string();

        let (h1, exited1) = dummy_handle();
        let (h2, exited2) = dummy_handle();

        register_handle(output.clone(), h1);
        assert!(!*exited1.lock().unwrap(), "first handle still alive");

        // Registering h2 should kill h1.
        register_handle(output.clone(), h2);
        assert!(*exited1.lock().unwrap(), "first handle stopped on replacement");
        assert!(!*exited2.lock().unwrap(), "second handle still alive");

        // Clean up.
        kill_for_output(&output);
        assert!(*exited2.lock().unwrap(), "second handle stopped by kill_for_output");
    }

    #[test]
    fn kill_for_output_is_idempotent() {
        // Killing a non-existent output should not panic.
        kill_for_output("__test_nonexistent_output__");
    }

    #[test]
    fn kill_all_stops_all_handles() {
        let out_a = "__test_kill_all_a__".to_string();
        let out_b = "__test_kill_all_b__".to_string();

        let (ha, exited_a) = dummy_handle();
        let (hb, exited_b) = dummy_handle();

        register_handle(out_a.clone(), ha);
        register_handle(out_b.clone(), hb);

        kill_all();

        assert!(*exited_a.lock().unwrap(), "handle A stopped");
        assert!(*exited_b.lock().unwrap(), "handle B stopped");
    }
}
