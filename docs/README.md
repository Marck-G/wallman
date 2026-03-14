# Wallman Documentation

Welcome to the official documentation for **Wallman**, the dynamic wallpaper manager for Wayland.

## Documentation Modules

### 📖 [User Manual](./user_manual.md)

Learn how to install, configure, and use Wallman day-to-day. Covers installation, the configuration file format, fill modes, daemon management, and the full CLI reference.

### 🎨 [Theme Building Guide](./theme_build.md)

A guide for theme creators. Learn how to structure a theme directory, write a `manifest.toml`, set fill modes, and distribute your work as a `.wallman` pack.

---

## Quick Links

- [Project GitHub](https://github.com/Marck-G/wallman)
- [Issue Tracker](https://github.com/Marck-G/wallman/issues)

---

## CLI Overview

| Command Group | Description |
|---------------|-------------|
| `wallman daemon` | Start, stop, restart, and check the background updater process |
| `wallman theme` | Install, activate, create, pack, and remove themes |
| `wallman config` | Manage global settings (lat, lon, day range, fill mode, …) |
| `wallman pack` | Build or inspect `.wallman` pack files |
| `wallman completion` | Generate and install shell autocompletion scripts |

## Key Concepts at a Glance

| Concept | Summary |
|---------|---------|
| **Renderer** | Native Wayland layer-shell (`zwlr_layer_shell_v1`) — no `swaybg` required |
| **Fill Mode** | Global image-scaling strategy: `fill` \| `crop` \| `fit` \| `scale` \| `tile` |
| **Triggers** | Priority order — Weather > DayTime > Static |
| **Outputs** | Per-monitor config with `"*"` wildcard support |
| **Themes** | Self-contained `.wallman` packs (tar + zstd) |
