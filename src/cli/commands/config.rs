use clap::Subcommand;

/// Subcommands for `wallman config`
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Create a default config file if none exists
    Init,

    /// Open the config file in $EDITOR
    Edit,

    /// Parse and validate the current config file
    Validate,

    /// Print the path to the active config file
    Path,
}
