use clap::{Parser, Subcommand};

use super::commands::{CompletionCommand, ConfigCommand, DaemonCommand, PackCommand, ThemeCommand};

/// Wallman — dynamic wallpaper manager for Sway / wlroots compositors
#[derive(Debug, Parser)]
#[command(
    name = "wallman",
    version,
    author,
    about = "Dynamic wallpaper manager for Sway and wlroots compositors",
    long_about = None,
    propagate_version = true,
)]
pub struct Cli {
    /// Top-level subcommand
    #[command(subcommand)]
    pub command: Command,

    /// Enable verbose operational logging
    #[arg(global = true, short, long)]
    pub verbose: bool,

    /// Enable debug-level tracing output
    #[arg(global = true, long)]
    pub debug: bool,
}

/// Top-level commands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manage wallpaper themes (.wallman packs)
    Theme {
        #[command(subcommand)]
        sub: ThemeCommand,
    },

    /// Control the wallman background daemon
    Daemon {
        #[command(subcommand)]
        sub: DaemonCommand,
    },

    /// Manage wallman configuration
    Config {
        #[command(subcommand)]
        sub: ConfigCommand,
    },

    /// Build or inspect .wallman pack files
    Pack {
        #[command(subcommand)]
        sub: PackCommand,
    },

    /// Generate shell completion scripts
    Completion {
        #[command(subcommand)]
        sub: CompletionCommand,
    },
}
