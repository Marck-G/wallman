# Wallman — Dynamic Wallpaper Manager for Wayland

**Wallman** is a powerful, dynamic wallpaper management system for **Wayland** compositors. It automates your desktop background based on time, weather, and custom themes, with native multi-monitor support and a built-in Wayland layer-shell renderer — no external tools required.

## ✨ Features

- 🕒 **Time-aware**: Automatically switch between day and night wallpapers based on configurable time ranges.
- 🌤️ **Weather-aware**: Reactive wallpapers based on local weather (Clear, Cloudy, Rainy, Snowy, Stormy).
- 🖥️ **Multi-monitor**: Assign different wallpapers per output or use wildcards (`*`) for all monitors.
- 🎨 **Theme System**: Install, pack, and share self-contained theme packs (`.wallman`).
- 🔄 **Exclusive Triggers**: Smart priority system prevents conflicts (Weather > Time > Static).
- 🖼️ **Flexible Fill Modes**: `fill`, `fit`, `scale`, `tile`, `crop` — one global setting, applied everywhere.
- 🧱 **Native Wayland Renderer**: Uses `zwlr_layer_shell_v1` directly — no `swaybg` dependency.
- 📦 **Multi-Arch**: Ready for x86_64, ARM64, and Raspberry Pi (ARMv7).
- 🛠️ **Developer Friendly**: Clean CLI with shell completion for Bash, Zsh, Fish, PowerShell, and Elvish.

---

## 📚 Documentation

Detailed guides are available in the [docs/](./docs/README.md) folder:

- 📖 **[User Manual](./docs/user_manual.md)**: Installation, usage, and configuration.
- 🎨 **[Theme Building](./docs/theme_build.md)**: Create and package your own themes.

---

## 🚀 Getting Started

### Prerequisites

- A **Wayland compositor** that supports `zwlr_layer_shell_v1` (Sway, Hyprland, river, and most wlroots-based compositors).
- **Rust** to build from source.

> **Note:** `swaybg` is no longer required. Wallman renders wallpapers natively via the Wayland layer-shell protocol.

### Installation

```bash
cargo build --release
sudo cp target/release/wallman /usr/local/bin/
```

### Shell Completion

```bash
# Install completion for your current shell
wallman completion install

# Force overwrite an existing completion file
wallman completion install --force

# Generate a script for a specific shell
wallman completion generate zsh > ~/.zsh/completions/_wallman
```

**Supported shells:** Bash, Zsh, Fish, PowerShell, Elvish

---

## 🎮 Usage

### 1. Daemon Management

```bash
wallman daemon start      # Start the daemon
wallman daemon stop       # Stop the daemon
wallman daemon restart    # Restart (picks up config changes)
wallman daemon status     # Check if it's running
```

### 2. Theme Management

```bash
wallman theme list                    # List installed themes
wallman theme set <name>              # Activate a theme
wallman theme install <file.wallman>  # Install a theme pack
wallman theme create <path>           # Scaffold a new theme directory
wallman theme remove <name>           # Delete an installed theme
```

### 3. Configuration

```bash
wallman config init                   # Create a default config
wallman config edit                   # Open config in $EDITOR
wallman config validate               # Check for syntax errors
wallman config path                   # Show path to config.toml

# Tune specific settings
wallman config set-lat 40.7128
wallman config set-lon -74.0060
wallman config set-day-range 06-18
wallman config set-fill-mode fit      # fill | crop | fit | scale | tile
```

---

## 🛠️ Configuration Guide

The global configuration lives at `~/.config/wallman/config.toml`.

### Fill Modes

The `fillMode` field is set once at the top level and applies to every output and every trigger:

| Mode    | Behaviour |
|---------|-----------|
| `fill`  | Scale to cover the output, centre-crop any overflow *(default)* |
| `crop`  | Alias for `fill` |
| `fit`   | Scale to fit inside the output, letterbox with black bars |
| `scale` | Stretch to exact output dimensions (ignores aspect ratio) |
| `tile`  | Repeat the image at its original size |

### Basic Example

```toml
version = 1
fillMode = "fill"

[background."*"]
image = "/path/to/default.jpg"
```

### Per-output backgrounds

```toml
fillMode = "fit"

[background."HDMI-A-1"]
image = "left-monitor.jpg"

[background."DP-1"]
image = "right-monitor.jpg"
```

### Time-based switching

```toml
fillMode = "fill"

[timeConfig."*"]
day   = "day.jpg"
night = "night.jpg"

dayRange = "06-18"
```

### Weather integration

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

## 🎨 Theme Development

A Wallman theme is a directory with:

```text
my-theme/
├── manifest.toml    # Theme config (triggers, images, fill mode)
└── images/          # Image assets
    ├── day.png
    └── night.png
```

Pack for distribution:

```bash
wallman theme pack ./my-theme   # → my-theme.wallman
```

---

## 🏗️ Architecture

- **Daemon**: Background service that monitors triggers and evaluates which wallpaper to apply.
- **Wayland Renderer**: Each output gets a dedicated thread with a `zwlr_layer_surface_v1` on the Background layer, rendered via shared-memory (`wl_shm`) buffers. Fully compatible with any wlr-layer-shell compositor.
- **AppState**: Global shared state managing the current configuration and theme pool.
- **OutputResolver**: Detects monitors via `swaymsg` (or `wl_output::Name`) and matches them to config keys.
- **Triggers**: Pluggable drivers (Static, DayTime, Weather) that emit `OutputChange` batches consumed by the renderer.

---

## 🚧 Roadmap

- [ ] Smooth transitions between wallpapers.
- [ ] Graphical user interface (GUI).
- [ ] Support for video/animated wallpapers.
- [ ] Per-output fill mode overrides.

---

## 📄 License

MIT © 2026 Wallman Contributors
