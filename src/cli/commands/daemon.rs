use clap::Subcommand;

/// Subcommands for `wallman daemon`
#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    /// Start the wallman background daemon
    Start {
        /// Run in the foreground instead of detaching
        #[arg(long)]
        foreground: bool,
    },

    /// Stop the running daemon
    Stop,

    /// Restart the daemon (stop + start)
    Restart,

    /// Show daemon status (running / stopped + PID)
    Status,
}
