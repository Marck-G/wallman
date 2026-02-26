# Wallman — Dynamic Wallpaper Manager for Sway

**Wallman** is a wallpaper management system designed specifically for **Sway / Wayland** environments.  
Its goal is to provide a flexible, extensible, and automation-focused way to control wallpapers based on context rather than static configuration.

Instead of treating wallpapers as a single fixed image, Wallman allows backgrounds to adapt dynamically to time, seasons, and environmental conditions.

> ⚠️ **Project Status:**  
> Wallman is currently under active construction.  
> Features, configuration structures, and internal APIs may change as development progresses.

---

## ✨ Core Concepts

Wallman separates wallpaper **selection logic** from **rendering**, allowing different strategies to decide which background should be displayed at any moment.

The project currently uses `swaybg` as its rendering backend, with plans to migrate toward a native layer-shell renderer in the future.

---

## 🌄 Supported Wallpaper Modes

Wallman can be configured using several high-level behaviors:

### • Static Wallpaper
A single fixed background applied consistently.

### • Day / Night Wallpapers
Automatically switches wallpapers depending on the time of day, enabling different visuals for daytime and nighttime environments.

### • Seasonal Wallpapers
Allows wallpapers to change according to seasonal context, enabling long-term automatic variation without user interaction.

### • Weather-Based Wallpapers
Wallpapers can react dynamically to real-world weather conditions using geographic location data. Different images may be displayed depending on detected weather states.

---

## 🧠 Configuration Philosophy

Configuration is designed to be:

- Declarative
- Context-aware
- Extensible
- Backend-independent

Rather than defining exact commands, users describe *conditions*, and Wallman determines which wallpaper should be active.

The internal configuration model supports:

- Optional static images
- Time-based switching
- Weather-driven image selection
- Flexible fill modes
- Future extensibility for additional triggers

---

## 🏗️ Architecture Goals

Wallman is being developed around a modular architecture:

- **Core Engine** — decides which wallpaper should be active
- **Backends** — responsible for rendering (currently `swaybg`)
- **Daemon Layer** — handles monitoring, updates, and system events
- **Future UI Layers** — CLI and graphical interfaces

This separation allows rendering systems to evolve without changing configuration logic.

---

## 🚀 Planned Features

- Monitor-aware wallpaper management
- Automatic output detection
- Background daemon
- Weather integration
- Time and seasonal automation
- IPC interface
- Native Wayland layer-shell renderer
- Animated transitions (future)

---

## 🎯 Project Vision

Wallman aims to become a native Wayland wallpaper manager focused on automation, adaptability, and clean architecture — designed for modern Sway workflows rather than legacy desktop assumptions.

---

## 🚧 Development Status

This project is experimental and evolving rapidly.  
Breaking changes are expected while core concepts and architecture stabilize.

Contributions, ideas, and feedback are welcome.