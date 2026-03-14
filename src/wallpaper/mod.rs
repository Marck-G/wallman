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
