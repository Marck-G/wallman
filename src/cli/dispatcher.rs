use std::{fs, path::PathBuf, process};

use crate::{
    Config,
    cli::{
        app::Command,
        commands::{CompletionCommand, ConfigCommand, DaemonCommand, PackCommand, ThemeCommand},
    },
    constants::{config_folder, decompresion_folder},
    daemon::DaemonManager,
    format::{install::PackInstaller, pack::Packager},
};
use clap::CommandFactory;

/// Exit codes defined by the plan (§10).
pub enum ExitCode {
    Success = 0,
    Error = 1,
    InvalidConfig = 2,
    PackError = 3,
    DaemonError = 4,
}

/// Route a parsed `Command` to the appropriate service function.
///
/// This function must **not** contain any filesystem or business logic itself —
/// it only orchestrates calls to service modules.
pub fn dispatch(command: Command) -> Result<(), (String, ExitCode)> {
    match command {
        Command::Theme { sub } => dispatch_theme(sub),
        Command::Daemon { sub } => dispatch_daemon(sub),
        Command::Config { sub } => dispatch_config(sub),
        Command::Pack { sub } => dispatch_pack(sub),
        Command::Completion { sub } => dispatch_completion(sub),
    }
}

// ── Theme ─────────────────────────────────────────────────────────────────────

fn dispatch_theme(cmd: ThemeCommand) -> Result<(), (String, ExitCode)> {
    match cmd {
        ThemeCommand::Create { path, name } => theme_create(path, name),
        ThemeCommand::Pack { path, output } => theme_pack(path, output),
        ThemeCommand::Install { file } => theme_install(file),
        ThemeCommand::List => theme_list(),
        ThemeCommand::Set { name } => theme_set(name),
        ThemeCommand::Remove { name } => theme_remove(name),
    }
}

fn theme_create(path: String, name: Option<String>) -> Result<(), (String, ExitCode)> {
    let dir = PathBuf::from(&path);
    let theme_name = name.unwrap_or_else(|| {
        dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-theme")
            .to_string()
    });

    // Create directory layout.
    let images_dir = dir.join("images");
    fs::create_dir_all(&images_dir).map_err(|e| {
        (
            format!("Failed to create theme directory: {e}"),
            ExitCode::Error,
        )
    })?;

    // Write a minimal manifest.toml (serialized from Config::default()).
    let mut default_cfg = Config::default();
    default_cfg.name = Some(theme_name.clone());
    default_cfg.description = Some("A wallman theme".to_string());

    let manifest_path = dir.join("manifest.toml");
    default_cfg.save_to_file(&manifest_path).map_err(|e| {
        (
            format!("Failed to write manifest.toml: {e}"),
            ExitCode::Error,
        )
    })?;

    println!("Theme '{}' created at {}", theme_name, dir.display());
    println!(
        "  Place your wallpaper images inside:  {}",
        images_dir.display()
    );
    println!(
        "  Edit the manifest:                   {}",
        manifest_path.display()
    );
    Ok(())
}

fn theme_pack(path: String, output: Option<String>) -> Result<(), (String, ExitCode)> {
    let dir = PathBuf::from(&path);

    // Load the manifest to get the theme name.
    let manifest_path = dir.join("manifest.toml");
    let config = Config::load(manifest_path.clone()).map_err(|e| {
        (
            format!("Error: manifest.toml not found or invalid: {e}"),
            ExitCode::PackError,
        )
    })?;

    let out_path = output.map(PathBuf::from).unwrap_or_else(|| {
        let stem = config.name.clone().unwrap_or_else(|| "theme".to_string());
        PathBuf::from(format!("{}.wallman", stem.replace(" ", "-")))
    });

    let packager = Packager::new(config, &dir);
    packager
        .pack(&out_path)
        .map_err(|e| (format!("Pack error: {e}"), ExitCode::PackError))?;

    println!("Theme packed → {}", out_path.display());
    Ok(())
}

fn theme_install(file: String) -> Result<(), (String, ExitCode)> {
    let mut installer = PackInstaller::new(&file);
    installer
        .install()
        .map_err(|e| (format!("Error: {e}"), ExitCode::PackError))?;

    println!("Theme installed successfully from {}", file);
    Ok(())
}

fn theme_list() -> Result<(), (String, ExitCode)> {
    let themes_dir = decompresion_folder();

    if !themes_dir.exists() {
        println!("No themes installed. ({})", themes_dir.display());
        return Ok(());
    }

    let mut count = 0usize;
    for entry in fs::read_dir(&themes_dir).map_err(|e| {
        (
            format!("Cannot read themes directory: {e}"),
            ExitCode::Error,
        )
    })? {
        let entry = entry.map_err(|e| (format!("{e}"), ExitCode::Error))?;
        let meta = entry
            .metadata()
            .map_err(|e| (format!("{e}"), ExitCode::Error))?;
        if meta.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Try to read the theme's manifest for description.
            let manifest = entry.path().join("manifest.toml");
            let description = Config::load(manifest)
                .ok()
                .and_then(|c| c.description)
                .unwrap_or_default();

            if description.is_empty() {
                println!("  {}", name);
            } else {
                println!("  {}  —  {}", name, description);
            }
            count += 1;
        }
    }

    if count == 0 {
        println!("No themes installed.");
    }

    Ok(())
}

fn theme_set(name: String) -> Result<(), (String, ExitCode)> {
    let theme_dir = decompresion_folder().join(&name);
    if !theme_dir.exists() {
        return Err((
            format!(
                "Error: theme '{}' is not installed. Run `wallman theme list` to see available themes.",
                name
            ),
            ExitCode::Error,
        ));
    }

    // Update the user config to point at this theme.
    let state_arc = crate::APP_STATE.get().unwrap().clone();
    let mut state = state_arc.lock().unwrap();
    state.config.pool = Some(theme_dir.to_string_lossy().to_string());
    state.save_config().map_err(|e| {
        (
            format!("Error: could not save config: {e}"),
            ExitCode::InvalidConfig,
        )
    })?;
    drop(state);

    println!("Active theme set to '{}'.", name);
    println!("Run `wallman daemon restart` for the change to take effect.");
    Ok(())
}

fn theme_remove(name: String) -> Result<(), (String, ExitCode)> {
    let theme_dir = decompresion_folder().join(&name);
    if !theme_dir.exists() {
        return Err((
            format!("Error: theme '{}' is not installed.", name),
            ExitCode::Error,
        ));
    }

    fs::remove_dir_all(&theme_dir).map_err(|e| {
        (
            format!("Error removing theme '{}': {e}", name),
            ExitCode::Error,
        )
    })?;

    println!("Theme '{}' removed.", name);
    Ok(())
}

// ── Daemon ────────────────────────────────────────────────────────────────────

fn dispatch_daemon(cmd: DaemonCommand) -> Result<(), (String, ExitCode)> {
    let dm = DaemonManager::new();
    match cmd {
        DaemonCommand::Start { foreground } => dm
            .start(foreground)
            .map_err(|e| (format!("Error: {e}"), ExitCode::DaemonError)),
        DaemonCommand::Stop => dm
            .stop()
            .map_err(|e| (format!("Error: {e}"), ExitCode::DaemonError)),
        DaemonCommand::Restart => dm
            .restart()
            .map_err(|e| (format!("Error: {e}"), ExitCode::DaemonError)),
        DaemonCommand::Status => dm
            .status()
            .map_err(|e| (format!("Error: {e}"), ExitCode::DaemonError)),
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

fn dispatch_config(cmd: ConfigCommand) -> Result<(), (String, ExitCode)> {
    match cmd {
        ConfigCommand::Init => config_init(),
        ConfigCommand::Edit => config_edit(),
        ConfigCommand::Validate => config_validate(),
        ConfigCommand::Path => config_path(),
    }
}

fn config_init() -> Result<(), (String, ExitCode)> {
    let cfg_path = config_folder().join("config.toml");

    if cfg_path.exists() {
        println!("Config already exists at {}", cfg_path.display());
        return Ok(());
    }

    Config::default().save_to_file(&cfg_path).map_err(|e| {
        (
            format!("Error: could not write config: {e}"),
            ExitCode::Error,
        )
    })?;

    println!("Config initialised at {}", cfg_path.display());
    Ok(())
}

fn config_edit() -> Result<(), (String, ExitCode)> {
    let cfg_path = config_folder().join("config.toml");

    // Ensure the file exists first.
    if !cfg_path.exists() {
        config_init()?;
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());

    let status = process::Command::new(&editor)
        .arg(&cfg_path)
        .status()
        .map_err(|e| {
            (
                format!("Error: could not launch editor '{}': {e}", editor),
                ExitCode::Error,
            )
        })?;

    if !status.success() {
        return Err((
            format!("Editor '{}' exited with non-zero status.", editor),
            ExitCode::Error,
        ));
    }

    Ok(())
}

fn config_validate() -> Result<(), (String, ExitCode)> {
    let cfg_path = config_folder().join("config.toml");

    if !cfg_path.exists() {
        return Err((
            format!(
                "Error: config not found at {}. Run `wallman config init` to create one.",
                cfg_path.display()
            ),
            ExitCode::InvalidConfig,
        ));
    }

    Config::load(cfg_path).map_err(|e| {
        (
            format!("Error: invalid config — {e}"),
            ExitCode::InvalidConfig,
        )
    })?;

    println!("Config is valid.");
    Ok(())
}

fn config_path() -> Result<(), (String, ExitCode)> {
    let cfg_path = config_folder().join("config.toml");
    println!("{}", cfg_path.display());
    Ok(())
}

// ── Pack ──────────────────────────────────────────────────────────────────────

fn dispatch_pack(cmd: PackCommand) -> Result<(), (String, ExitCode)> {
    match cmd {
        PackCommand::Build { path, output } => theme_pack(path, output),
        PackCommand::Inspect { file } => pack_inspect(file),
    }
}

fn pack_inspect(file: String) -> Result<(), (String, ExitCode)> {
    use std::fs::File;
    use tar::Archive;
    use zstd::Decoder;

    let f = File::open(&file).map_err(|e| {
        (
            format!("Error: cannot open '{}': {e}", file),
            ExitCode::PackError,
        )
    })?;

    let decoder = Decoder::new(f).map_err(|e| (format!("Error: {e}"), ExitCode::PackError))?;
    let mut archive = Archive::new(decoder);

    println!("Contents of {}:", file);
    println!("{:<50}  {}", "Entry", "Size (bytes)");
    println!("{}", "-".repeat(62));

    for entry in archive
        .entries()
        .map_err(|e| (format!("Error reading pack: {e}"), ExitCode::PackError))?
    {
        let entry = entry.map_err(|e| (format!("{e}"), ExitCode::PackError))?;
        let path = entry
            .path()
            .map_err(|e| (format!("{e}"), ExitCode::PackError))?;
        let size = entry.size();
        println!("{:<50}  {}", path.display(), size);
    }

    Ok(())
}

// ── Completion ────────────────────────────────────────────────────────────────

fn dispatch_completion(cmd: CompletionCommand) -> Result<(), (String, ExitCode)> {
    match cmd {
        CompletionCommand::Generate { shell } => {
            let mut cmd = crate::cli::app::Cli::command();
            crate::cli::commands::completion::generate_completion(shell, &mut cmd)
                .map_err(|e| (format!("Error generating completion: {e}"), ExitCode::Error))
        }
        CompletionCommand::Install { force } => {
            crate::cli::commands::completion::install_completion(force)
                .map_err(|e| (format!("Error installing completion: {e}"), ExitCode::Error))
        }
        CompletionCommand::Uninstall => crate::cli::commands::completion::uninstall_completion()
            .map_err(|e| {
                (
                    format!("Error uninstalling completion: {e}"),
                    ExitCode::Error,
                )
            }),
    }
}
