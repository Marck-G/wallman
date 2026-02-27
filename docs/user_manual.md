# Wallman User Manual

Wallman is a dynamic wallpaper manager designed for Sway and other wlroots-based compositors. It supports multiple monitors, time-based switching, and real-time weather integration.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Configuration](#configuration)
5. [Daemon Management](#daemon-management)
6. [CLI Reference](#cli-reference)

---

## Core Concepts

- **Daemon**: A background process that monitors time or weather and updates your wallpaper accordingly.
- **Triggers**: Drivers for wallpaper changes. Wallman uses a priority system:
    1. **Weather Trigger**: Highest priority. Changes based on live API data.
    2. **DayTime Trigger**: Changes based on the time of day.
    3. **Static Trigger**: Fallback. Sets a consistent wallpaper on startup.
- **Themes**: Packaged collections of images and configs (`.wallman` files).
- **Output Resolution**: Wallman automatically detects your monitors (e.g., `DP-1`, `HDMI-A-1`) and applies specific settings to each, including wildcard (`*`) support.

---

## Installation

### Dependencies

- `swaybg`: Required for actually setting the wallpaper.
- `zstd`: Required for theme decompression.

### Building from Source

```bash
cargo build --release
sudo cp target/release/wallman /usr/local/bin/
```

### Shell Completion

To enable autocompletion for your shell:

```bash
wallman completion install
```

---

## Quick Start

1. Initialize your config: `wallman config init`
2. Start the daemon: `wallman daemon start`
3. Download or create a theme and install it: `wallman theme install my-theme.wallman`
4. Set the theme as active: `wallman theme set my-theme`

---

## Configuration

The configuration file is located at `~/.config/wallman/config.toml`.

### Basic Background

```toml
[background."*"]
image = "/path/to/image.jpg"
fill_mode = "fill"
```

### Time-Based Switching

```toml
[timeConfig."*"]
day = "day-image.jpg"
night = "night-image.jpg"
day_range = "8-19" # Day starts at 8:00 and ends at 19:00
```

### Weather Integration

```toml
[weather."*"]
lat = 40.7128
lon = -74.0060

[weather."*".weather]
clear = "sunny.jpg"
cloudy = "vague.jpg"
rainy = "wet.jpg"

# Supports: clear, cloudy, rainy, snowy, stormy
```

---

## Daemon Management

The daemon must be running for dynamic updates to work.

- `wallman daemon start`: Starts the background process.
- `wallman daemon stop`: Gracefully stops the process.
- `wallman daemon status`: Checks if the daemon is running.
- `wallman daemon restart`: Restarts the daemon to reload config changes.

---

## CLI Reference

### Theme Commands

- `wallman theme list`: Show all installed themes.
- `wallman theme set <name>`: Switch to a specific installed theme.
- `wallman theme create <path>`: Scaffold a new theme directory.
- `wallman theme install <file.wallman>`: Install a theme package.

### Config Commands

- `wallman config path`: Show current config location.
- `wallman config edit`: Open config in your default editor.
- `wallman config init`: Create a default configuration.

### Completion Commands

- `wallman completion generate <shell>`: Output shell completion script.
- `wallman completion install`: Automatically install for current shell.
