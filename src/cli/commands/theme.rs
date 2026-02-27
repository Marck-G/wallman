use clap::Subcommand;

/// Subcommands for `wallman theme`
#[derive(Debug, Subcommand)]
pub enum ThemeCommand {
    /// Scaffold a new theme directory at <path>
    Create {
        /// Target directory for the new theme
        path: String,
        /// Optional theme name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Package a theme directory into a .wallman file
    Pack {
        /// Source theme directory
        path: String,
        /// Output .wallman file path (default: <name>.wallman)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Install a .wallman pack file
    Install {
        /// Path to the .wallman file
        file: String,
    },

    /// List all installed themes
    List,

    /// Activate a theme by name
    Set {
        /// Theme name as shown by `wallman theme list`
        name: String,
    },

    /// Remove an installed theme
    Remove {
        /// Theme name to remove
        name: String,
    },
}
