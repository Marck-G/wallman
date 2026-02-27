use clap::Parser;
use std::{
    path::PathBuf,
    process,
    sync::{Arc, Mutex},
};

use tracing_subscriber::{EnvFilter, fmt};

use wallman::{
    APP_STATE, AppState, Config,
    cli::{Cli, dispatch},
    constants::{config_folder, config_vec},
};

fn main() {
    // ── 1. Parse CLI ─────────────────────────────────────────────────────
    let cli = Cli::parse();

    // ── 2. Initialise tracing / logging ─────────────────────────────────
    init_logging(cli.verbose, cli.debug);

    // ── 3. Bootstrap APP_STATE ───────────────────────────────────────────
    if let Err(e) = init_app_state() {
        eprintln!("Error: failed to load configuration — {e}");
        process::exit(2);
    }

    // ── 4. Dispatch command ──────────────────────────────────────────────
    match dispatch(cli.command) {
        Ok(()) => process::exit(0),
        Err((msg, code)) => {
            eprintln!("{}", msg);
            process::exit(code as i32);
        }
    }
}

/// Initialise tracing-subscriber based on verbosity flags.
fn init_logging(verbose: bool, debug: bool) {
    let filter = if debug {
        "wallman=debug,warn"
    } else if verbose {
        "wallman=info,warn"
    } else {
        "warn"
    };

    fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .with_thread_ids(false)
        .compact()
        .init();
}

/// Load config and initialise the global APP_STATE.
///
/// Tries each path returned by `config_vec()` in order.
/// Falls back to `Config::default()` if none are found.
fn init_app_state() -> Result<(), Box<dyn std::error::Error>> {
    let config_path_resolved: PathBuf;
    let config: Config;

    // Try user config locations in priority order.
    let candidates: Vec<PathBuf> = config_vec();
    let found = candidates
        .iter()
        .find(|p| p.with_extension("toml").exists());

    if let Some(path) = found {
        let toml_path = path.with_extension("toml");
        config = Config::load(toml_path.clone())?;
        config_path_resolved = toml_path;
        tracing::info!("Loaded config from {}", config_path_resolved.display());
    } else {
        tracing::info!("No config found — using defaults");
        config = Config::default();
        config_path_resolved = config_folder().join("config.toml");
    }

    // If a theme pool is active, merge its manifest settings.
    let mut config = config;
    if let Some(pool) = &config.pool {
        let pool_path = PathBuf::from(pool);
        if let Err(e) = config.merge_theme(pool_path) {
            tracing::warn!("Failed to merge theme manifest: {}", e);
        }
    }

    let images_pool = config.pool.clone();
    let is_pool = images_pool.is_some();

    let state = AppState::new(
        config,
        config_path_resolved.to_string_lossy().to_string(),
        images_pool,
        is_pool,
    )?;

    APP_STATE
        .set(Arc::new(Mutex::new(state)))
        .map_err(|_| "APP_STATE already initialised")?;

    Ok(())
}
