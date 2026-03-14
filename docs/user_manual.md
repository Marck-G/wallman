# Wallman User Manual

Wallman is a dynamic wallpaper manager for Wayland compositors. It supports multiple monitors, time-based switching, real-time weather integration, and a native Wayland renderer — no external tools required.

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Configuration](#configuration)
5. [Fill Modes](#fill-modes)
6. [Daemon Management](#daemon-management)
7. [CLI Reference](#cli-reference)

---

## Core Concepts

- **Daemon**: A background process that monitors time or weather and updates your wallpaper accordingly.
- **Triggers**: Drivers for wallpaper changes. Wallman uses a priority system:
    1. **Weather Trigger** — Highest priority. Changes based on live weather API data.
    2. **DayTime Trigger** — Changes based on the current time of day.
    3. **Static Trigger** — Fallback. Sets a consistent wallpaper on startup.
- **Fill Mode**: A single global setting that controls how images are scaled to fit each output. Set it once in `config.toml` or via `wallman config set-fill-mode`.
- **Themes**: Packaged collections of images and a config manifest (`.wallman` files).
- **Output Resolution**: Wallman detects your monitors (e.g., `DP-1`, `HDMI-A-1`) and applies settings per output, with wildcard (`*`) support for all monitors at once.

---

## Installation

### Prerequisites

- A **Wayland compositor** that supports `zwlr_layer_shell_v1`:
  - Sway, Hyprland, river, labwc, and other wlroots-based compositors.
- **Rust** toolchain to build from source.

> `swaybg` is **no longer required**. Wallman renders wallpapers natively using the Wayland layer-shell protocol (`zwlr_layer_shell_v1`) via shared-memory buffers.

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

Restart your shell or source the generated file to activate it.

---

## Quick Start

```bash
# 1. Create a default config
wallman config init

# 2. Start the daemon
wallman daemon start

# 3. Install a theme
wallman theme install my-theme.wallman

# 4. Activate the theme
wallman theme set my-theme
```

---

## Configuration

The configuration file is located at `~/.config/wallman/config.toml`.

### Top-level fields

| Field       | Type     | Description |
|-------------|----------|-------------|
| `version`   | integer  | Config format version (currently `1`) |
| `pool`      | string   | Path to the active theme directory |
| `fillMode`  | string   | Global fill mode for all outputs (see [Fill Modes](#fill-modes)) |
| `lat`       | float    | Latitude for weather/daytime triggers |
| `lon`       | float    | Longitude for weather/daytime triggers |
| `dayRange`  | string   | Daytime window in `HH-HH` format (e.g. `"06-18"`) |

### Static Background

```toml
version = 1
fillMode = "fill"

[background."*"]
image = "/path/to/image.jpg"
```

Use a specific output name instead of `"*"` for per-monitor images:

```toml
[background."HDMI-A-1"]
image = "left.jpg"

[background."DP-1"]
image = "right.jpg"
```

### Time-Based Switching

```toml
fillMode = "fit"
dayRange = "06-18"   # 6 AM → 6 PM is "day"

[timeConfig."*"]
day   = "day.jpg"
night = "night.jpg"
```

### Weather Integration

```toml
lat = 40.7128
lon = -74.0060
fillMode = "fill"

[weather."*".weather]
clear   = "sunny.jpg"
cloudy  = "overcast.png"
rainy   = "rain.jpg"
snowy   = "snow.jpg"
stormy  = "storm.jpg"
```

---

## Fill Modes

The `fillMode` field is a **global setting** — it applies to every output regardless of which trigger fires. Set it in `config.toml` or via the CLI:

```bash
wallman config set-fill-mode <mode>
```

| Mode    | Behaviour |
|---------|-----------|
| `fill`  | Scale to cover the entire output, centre-crop any overflow *(default)* |
| `crop`  | Alias for `fill` (backwards compatibility) |
| `fit`   | Scale to fit inside the output, preserving aspect ratio — letterboxes with black bars |
| `scale` | Stretch to exact output dimensions, ignoring aspect ratio |
| `tile`  | Repeat the image at its original size across the output |

**Example:**

```toml
# config.toml
fillMode = "tile"
```

```bash
# or via CLI
wallman config set-fill-mode fit
wallman daemon restart
```

---

## Daemon Management

The daemon must be running for dynamic updates to work.

```bash
wallman daemon start     # Start the background process
wallman daemon stop      # Gracefully stop the process
wallman daemon restart   # Restart and reload config changes
wallman daemon status    # Check whether the daemon is running
```

---

## CLI Reference

### `wallman config`

| Subcommand                        | Description |
|-----------------------------------|-------------|
| `config init`                     | Create a default `config.toml` |
| `config edit`                     | Open config in `$EDITOR` |
| `config validate`                 | Parse and validate the config file |
| `config path`                     | Print the path to `config.toml` |
| `config set-lat <value>`          | Set latitude (−90 to 90) |
| `config set-lon <value>`          | Set longitude (−180 to 180) |
| `config set-day-range <HH-HH>`    | Set the daytime window (e.g. `06-18`) |
| `config set-fill-mode <mode>`     | Set global fill mode (`fill` \| `crop` \| `fit` \| `scale` \| `tile`) |

### `wallman theme`

| Subcommand                            | Description |
|---------------------------------------|-------------|
| `theme list`                          | Show all installed themes |
| `theme set <name>`                    | Activate an installed theme |
| `theme create <path>`                 | Scaffold a new theme directory |
| `theme install <file.wallman>`        | Install a theme pack |
| `theme remove <name>`                 | Delete an installed theme |
| `theme pack <path>`                   | Pack a theme directory into `.wallman` |

### `wallman daemon`

| Subcommand          | Description |
|---------------------|-------------|
| `daemon start`      | Start the daemon |
| `daemon stop`       | Stop the daemon |
| `daemon restart`    | Restart the daemon |
| `daemon status`     | Check daemon status |

### `wallman completion`

| Subcommand                        | Description |
|-----------------------------------|-------------|
| `completion generate <shell>`     | Output a completion script (bash/zsh/fish/powershell/elvish) |
| `completion install [--force]`    | Install completion for the current shell |
| `completion uninstall`            | Remove the installed completion file |

### `wallman pack`

| Subcommand                            | Description |
|---------------------------------------|-------------|
| `pack build <path>`                   | Build a `.wallman` pack from a theme directory |
| `pack inspect <file.wallman>`         | List the contents of a pack file |
