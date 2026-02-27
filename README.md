# Wallman — Dynamic Wallpaper Manager for Sway

**Wallman** is a powerful, dynamic wallpaper management system designed specifically for **Sway / Wayland** environments. It allows you to automate your desktop background based on time, weather, and custom themes, with native support for multi-monitor setups.

## ✨ Features

- 🕒 **Time-aware**: Automatically switch between day and night wallpapers.
- 🌤️ **Weather-aware**: Reactive wallpapers based on local weather (Clear, Cloudy, Rainy, Snowy, Stormy).
- 🖥️ **Multi-monitor**: Assign different wallpapers to different monitors or use wildcards (`*`).
- 🎨 **Theme System**: Install, pack, and share self-contained theme packs (`.wallman`).
- 🔄 **Continuous Daemon**: Runs as a lightweight background service.
- 🛠️ **Developer Friendly**: Clean CLI for control and configuration.

---

## 🚀 Getting Started

### Prerequisites

- **Sway** (or a compatible Wayland compositor).
- **swaybg**: The current rendering backend.
- **Rust**: To build from source.

### Installation

```bash
cargo build --release
sudo cp target/release/wallman /usr/local/bin/
```

### Shell Completion

Wallman supports shell completion for better command-line experience. Install completions for your shell:

**Using the built-in command:**
```bash
# Generate and install completion for your current shell
wallman completion install

# Force overwrite existing completion
wallman completion install --force

# Generate completion for specific shell
wallman completion generate bash > ~/.local/share/bash-completion/completions/wallman
```

**Using the installation script:**
```bash
# Install completion for current shell
./scripts/install-completion.sh -i

# Show manual installation instructions
./scripts/install-completion.sh -m
```

**Supported shells:** Bash, Zsh, Fish, PowerShell, Elvish

After installation, restart your shell or source the completion file to enable autocompletion.

---

## 🎮 Usage

Wallman is controlled through a unified CLI.

### 1. Daemon Management
Launch the background process to start automating your wallpapers.

```bash
wallman daemon start      # Start the daemon
wallman daemon stop       # Stop the daemon
wallman daemon restart    # Restart the daemon
wallman daemon status     # Check if it's running
```

### 2. Theme Management
Wallman uses a unique theme system where configuration and images are bundled together.

```bash
wallman theme list                 # List installed themes
wallman theme set <name>           # Activate a theme
wallman theme install <file.wallman> # Install a new theme pack
wallman theme create <path>        # Scaffold a new theme directory
wallman theme remove <name>        # Delete an installed theme
```

### 3. Configuration
Manage your global settings.

```bash
wallman config init       # Create a default config
wallman config edit       # Open config in your $EDITOR
wallman config validate   # Check for syntax errors
wallman config path       # Show path to config.toml
```

---

## 🛠️ Configuration Guide

The global configuration is located at `~/.config/wallman/config.toml`.

### Basic Example
```toml
pool = "/home/user/.local/share/wallman/packs/themes/my-theme"
version = 1

[background."*"]
image = "default.png"
fillMode = "fill"
```

### Dynamic Wallpapers (Time based)
```toml
[timeConfig."*"]
day = "day_image.jpg"
night = "night_image.jpg"
```

### Weather Integration
Requires latitude and longitude for API lookups.
```toml
[weather."*"]
lat = 40.7128
lon = -74.0060

[weather."*".weather]
clear = "sunny.jpg"
cloudy = "overcast.png"
rainy = "rain.jpg"
snowy = "snow.jpg"
stormy = "storm.jpg"
```

---

## 🎨 Theme Development

A Wallman theme is a directory with the following structure:
```text
my-theme/
├── manifest.toml    # Theme settings (backgrounds, triggers)
└── images/          # Image assets
    ├── day.png
    └── night.png
```

### Packing a Theme
To share your theme, pack it into a `.wallman` file (Zstd-compressed tarball):
```bash
wallman theme pack ./my-theme
```

---

## 🏗️ Architecture

- **Daemon**: The core service that monitors triggers (Time, Weather) and evaluates which wallpaper should be active.
- **AppState**: Global shared state managing the current configuration and theme pool.
- **OutputResolver**: Detects monitors via `swaymsg` and matches them to configuration keys.
- **Backends**: Decoupled rendering logic (currently using `swaybg`).

---

## 🚧 Roadmap

- [ ] Native Layer-Shell renderer (removing `swaybg` dependency).
- [ ] Smooth transitions between wallpapers.
- [ ] Graphical user interface (GUI).
- [ ] Support for video/animated wallpapers.

---

## 📄 License

MIT © 2026 Wallman Contributors