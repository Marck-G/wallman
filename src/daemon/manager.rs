use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process,
};

use tracing::info;

/// Exit codes returned by daemon operations.
pub enum DaemonExitCode {
    Success = 0,
    AlreadyRunning = 1,
    NotRunning = 2,
    Error = 3,
}

/// Manages the wallman daemon process lifecycle via a PID file.
///
/// The PID file is stored at `<data_dir>/wallman/daemon.pid`.
/// The daemon itself is the `wallman daemon start --foreground` process; the
/// non-foreground path re-invokes the current executable with
/// `daemon start --foreground` and detaches via double-fork.
pub struct DaemonManager {
    pid_file: PathBuf,
}

impl DaemonManager {
    pub fn new() -> Self {
        let pid_file = crate::data_folder().join("daemon.pid");
        Self { pid_file }
    }

    // ── Public API ────────────────────────────────────────────────────────

    /// Start the daemon.
    ///
    /// If `foreground` is true, run the trigger loop directly in this process
    /// (used by the re-invoked child after double-fork).
    /// If false, spawn a detached child process and return immediately.
    pub fn start(&self, foreground: bool) -> Result<(), Box<dyn std::error::Error>> {
        if foreground {
            self.run_foreground()
        } else {
            if let Some(pid) = self.read_pid()? {
                if self.is_process_running(pid) {
                    return Err(format!(
                        "Daemon is already running (PID {pid}). \
                        Use `wallman daemon restart` to restart it."
                    )
                    .into());
                }
                // Stale PID file — remove it before re-spawning.
                let _ = fs::remove_file(&self.pid_file);
            }
            self.spawn_detached()
        }
    }

    /// Stop the daemon by sending SIGTERM to the stored PID.
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pid = self
            .read_pid()?
            .ok_or("Daemon is not running (no PID file found)")?;

        if !self.is_process_running(pid) {
            let _ = fs::remove_file(&self.pid_file);
            return Err(
                format!("No process found with PID {pid}. Cleaned up stale PID file.").into(),
            );
        }

        self.send_sigterm(pid)?;
        let _ = fs::remove_file(&self.pid_file);
        tracing::info!("Daemon (PID {}) stopped.", pid);
        Ok(())
    }

    /// Restart = stop (if running) then start.
    pub fn restart(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(pid) = self.read_pid()? {
            if self.is_process_running(pid) {
                self.send_sigterm(pid)?;
                // Brief wait for the process to exit.
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            let _ = fs::remove_file(&self.pid_file);
        }
        self.spawn_detached()
    }

    /// Print daemon status to stdout.
    pub fn status(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.read_pid()? {
            None => {
                println!("wallman daemon: stopped (no PID file)");
            }
            Some(pid) => {
                if self.is_process_running(pid) {
                    println!("wallman daemon: running  (PID {})", pid);
                } else {
                    println!("wallman daemon: stopped  (stale PID file for {})", pid);
                }
            }
        }
        Ok(())
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    /// Run the trigger loop in this process (foreground / child mode).
    fn run_foreground(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Write our own PID.
        self.write_pid(process::id())?;
        // Install SIGTERM handler to clean up the PID file on shutdown.
        #[cfg(unix)]
        {
            let pid_file = self.pid_file.clone();
            unsafe {
                nix::sys::signal::signal(
                    nix::sys::signal::Signal::SIGTERM,
                    nix::sys::signal::SigHandler::Handler(handle_sigterm),
                )
                .ok();
            }
            // Store for the signal handler (static).
            PID_FILE_PATH
                .set(pid_file)
                .expect("PID_FILE_PATH set twice");
        }

        info!("Daemon started in foreground (PID {})", process::id());

        // Build and run the trigger manager.
        let mut manager = build_trigger_manager()?;
        manager.run()?;

        Ok(())
    }

    /// Spawn a detached child that runs `wallman daemon start --foreground`.
    fn spawn_detached(&self) -> Result<(), Box<dyn std::error::Error>> {
        let exe = std::env::current_exe()?;
        info!("Spawning detached child");
        let child = std::process::Command::new(&exe)
            .args(&["daemon", "start", "--foreground"])
            // Detach stdio so the parent can exit cleanly.
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            // On Unix, create a new session so the child is not killed
            // when the terminal closes.
            .spawn()?;

        tracing::info!("Daemon spawned (PID {}).", child.id());
        println!("wallman daemon started (PID {}).", child.id());
        Ok(())
    }

    /// Read the PID stored in the PID file; returns None if file doesn't exist.
    fn read_pid(&self) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        if !self.pid_file.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&self.pid_file)?;
        let pid: u32 = contents.trim().parse()?;
        Ok(Some(pid))
    }

    /// Write a PID to the PID file (creates parent dirs if needed).
    fn write_pid(&self, pid: u32) -> io::Result<()> {
        if let Some(parent) = self.pid_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::File::create(&self.pid_file)?;
        write!(f, "{}", pid)?;
        Ok(())
    }

    /// Returns true if a process with the given PID currently exists.
    fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            use nix::sys::signal;
            use nix::unistd::Pid;
            signal::kill(Pid::from_raw(pid as i32), None).is_ok()
        }
        #[cfg(not(unix))]
        {
            // Fallback: check by opening /proc/<pid>
            std::path::Path::new(&format!("/proc/{}", pid)).exists()
        }
    }

    /// Send SIGTERM to a process by PID.
    fn send_sigterm(&self, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(format!("Cannot send SIGTERM on this platform (PID {})", pid).into())
        }
    }
}

// ── SIGTERM handler (Unix only) ───────────────────────────────────────────────

/// Static storage for the PID file path so the signal handler can clean up.
#[cfg(unix)]
static PID_FILE_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

#[cfg(unix)]
type LibcSig = libc::c_int;

#[cfg(unix)]
extern "C" fn handle_sigterm(_: LibcSig) {
    if let Some(path) = PID_FILE_PATH.get() {
        let _ = fs::remove_file(path);
    }
    process::exit(0);
}

// ── Trigger manager factory ───────────────────────────────────────────────────

/// Build the TriggerManager with all configured triggers, reading from APP_STATE.
fn build_trigger_manager()
-> Result<crate::triggers::manager::TriggerManager, Box<dyn std::error::Error>> {
    use crate::triggers::{
        daytime_trigger::DayTimeTrigger, manager::TriggerManager, static_trigger::StaticTrigger,
        weather_trigger::WeatherTrigger,
    };

    let state = crate::APP_STATE.get().unwrap().lock().unwrap();
    let config = state.config.clone();
    drop(state);

    let mut manager = TriggerManager::new();

    // Mutual Exclusive Trigger Selection (§17/Phase 2)
    // Priority: Weather > Time > Static
    if config.weather.is_some() {
        tracing::info!("Using WeatherTrigger (exclusive)");
        manager.add(Box::new(WeatherTrigger::new()));
    } else if config.time_config.is_some() {
        tracing::info!("Using DayTimeTrigger (exclusive)");
        manager.add(Box::new(DayTimeTrigger::new()));
    } else {
        tracing::info!("Using StaticTrigger (exclusive)");
        manager.add(Box::new(StaticTrigger::new()));
    }
    Ok(manager)
}
