use clap::{Command, CommandFactory, ValueEnum};
use clap_complete::{Shell, generate};
use std::io::{self, Write};

/// Supported shell types for completion generation
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

/// Subcommands for `wallman completion`
#[derive(Debug, clap::Subcommand)]
pub enum CompletionCommand {
    /// Generate completion script for the specified shell
    Generate {
        /// Shell to generate completion for
        shell: ShellType,
    },

    /// Install completion for the current shell
    Install {
        /// Force overwrite existing completion
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall completion for the current shell
    Uninstall,
}

/// Generate completion script for the specified shell
pub fn generate_completion(shell: ShellType, cmd: &mut Command) -> io::Result<()> {
    let mut buf = Vec::new();

    match shell {
        ShellType::Bash => generate(Shell::Bash, cmd, "wallman", &mut buf),
        ShellType::Zsh => generate(Shell::Zsh, cmd, "wallman", &mut buf),
        ShellType::Fish => generate(Shell::Fish, cmd, "wallman", &mut buf),
        ShellType::PowerShell => generate(Shell::PowerShell, cmd, "wallman", &mut buf),
        ShellType::Elvish => generate(Shell::Elvish, cmd, "wallman", &mut buf),
    }

    io::stdout().write_all(&buf)
}

/// Install completion for the current shell
pub fn install_completion(force: bool) -> io::Result<()> {
    let shell = detect_shell()?;
    let completion_dir = get_completion_dir(shell)?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&completion_dir)?;

    let completion_file = completion_dir.join(get_completion_filename(shell));

    if completion_file.exists() && !force {
        eprintln!(
            "Completion file already exists: {}",
            completion_file.display()
        );
        eprintln!("Use --force to overwrite");
        return Ok(());
    }

    // Generate completion script
    let mut cmd = crate::cli::app::Cli::command();
    let mut buf = Vec::new();

    match shell {
        ShellType::Bash => generate(Shell::Bash, &mut cmd, "wallman", &mut buf),
        ShellType::Zsh => generate(Shell::Zsh, &mut cmd, "wallman", &mut buf),
        ShellType::Fish => generate(Shell::Fish, &mut cmd, "wallman", &mut buf),
        ShellType::PowerShell => generate(Shell::PowerShell, &mut cmd, "wallman", &mut buf),
        ShellType::Elvish => generate(Shell::Elvish, &mut cmd, "wallman", &mut buf),
    }

    std::fs::write(&completion_file, buf)?;

    println!("Completion installed to: {}", completion_file.display());
    println!(
        "Restart your shell or run: source {}",
        completion_file.display()
    );

    Ok(())
}

/// Uninstall completion for the current shell
pub fn uninstall_completion() -> io::Result<()> {
    let shell = detect_shell()?;
    let completion_dir = get_completion_dir(shell)?;
    let completion_file = completion_dir.join(get_completion_filename(shell));

    if completion_file.exists() {
        std::fs::remove_file(&completion_file)?;
        println!("Completion uninstalled from: {}", completion_file.display());
    } else {
        println!("No completion file found at: {}", completion_file.display());
    }

    Ok(())
}

/// Detect the current shell
fn detect_shell() -> io::Result<ShellType> {
    let shell_path = std::env::var("SHELL")
        .or_else(|_| std::env::var("COMSPEC"))
        .or_else(|_| std::env::var("PSModulePath"))
        .unwrap_or_default();

    let shell_name = std::path::Path::new(&shell_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_lowercase();

    if shell_name.contains("bash") {
        Ok(ShellType::Bash)
    } else if shell_name.contains("zsh") {
        Ok(ShellType::Zsh)
    } else if shell_name.contains("fish") {
        Ok(ShellType::Fish)
    } else if shell_name.contains("powershell") || shell_name.contains("pwsh") {
        Ok(ShellType::PowerShell)
    } else {
        // Default to bash for unknown shells
        Ok(ShellType::Bash)
    }
}

/// Get the completion directory for the specified shell
fn get_completion_dir(shell: ShellType) -> io::Result<std::path::PathBuf> {
    match shell {
        ShellType::Bash => {
            // Try user-specific first, then system-wide
            let user_dir = dirs::home_dir()
                .map(|h| h.join(".local/share/bash-completion/completions"))
                .unwrap_or_default();

            if user_dir.exists() {
                Ok(user_dir)
            } else {
                Ok(std::path::PathBuf::from("/etc/bash_completion.d"))
            }
        }
        ShellType::Zsh => {
            let user_dir = dirs::home_dir()
                .map(|h| h.join(".zsh/completions"))
                .unwrap_or_default();

            if user_dir.exists() {
                Ok(user_dir)
            } else {
                Ok(std::path::PathBuf::from(
                    "/usr/local/share/zsh/site-functions",
                ))
            }
        }
        ShellType::Fish => Ok(dirs::home_dir()
            .map(|h| h.join(".config/fish/completions"))
            .unwrap_or_default()),
        ShellType::PowerShell => Ok(dirs::home_dir()
            .map(|h| h.join("Documents/PowerShell/Completion"))
            .unwrap_or_default()),
        ShellType::Elvish => Ok(dirs::home_dir()
            .map(|h| h.join(".config/elvish/lib"))
            .unwrap_or_default()),
    }
}

/// Get the completion filename for the specified shell
fn get_completion_filename(shell: ShellType) -> &'static str {
    match shell {
        ShellType::Bash => "wallman",
        ShellType::Zsh => "_wallman",
        ShellType::Fish => "wallman.fish",
        ShellType::PowerShell => "wallman.ps1",
        ShellType::Elvish => "wallman.elv",
    }
}
