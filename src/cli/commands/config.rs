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

    /// Set the latitude for location-based triggers (e.g., 40.7128)
    SetLat {
        /// Latitude value (-90 to 90)
        value: f64,
    },

    /// Set the longitude for location-based triggers (e.g., -74.0060)
    SetLon {
        /// Longitude value (-180 to 180)
        value: f64,
    },

    /// Set the day range for daytime triggers (e.g., "06-18" for 6 AM to 6 PM)
    SetDayRange {
        /// Day range in HH-HH format (e.g., "06-18")
        value: String,
    },
}
