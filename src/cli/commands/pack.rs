use clap::Subcommand;

/// Subcommands for `wallman pack`
#[derive(Debug, Subcommand)]
pub enum PackCommand {
    /// Build a .wallman pack from the given theme directory
    Build {
        /// Source theme directory
        path: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Inspect the contents of a .wallman pack without installing it
    Inspect {
        /// .wallman file to inspect
        file: String,
    },
}
