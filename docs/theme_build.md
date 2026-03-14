# Building Wallman Themes

A Wallman theme is a portable package that bundles wallpaper images and a configuration manifest describing how those images should be applied.

---

## Theme Structure

```text
my-theme/
├── manifest.toml    # Trigger config and metadata
└── images/          # Wallpaper image assets
    ├── day.jpg
    ├── night.jpg
    └── background.png
```

---

## The `manifest.toml`

The manifest is a standard Wallman configuration file. When a theme is activated, its settings drive the daemon. Only put image filenames here — not full paths. Wallman resolves them against the `images/` folder automatically.

### Minimal example (static wallpaper)

```toml
name        = "My Theme"
description = "A simple static wallpaper theme"
version     = 1
fillMode    = "fill"

[background."*"]
image = "background.png"
```

### Day / Night example

```toml
name        = "Moon Wall"
description = "A sleek cosmic theme that switches with the sun"
version     = 1
fillMode    = "fit"

[background."*"]
image = "background.png"

[timeConfig."*"]
day   = "day.jpg"
night = "night.jpg"
```

### Weather-driven example

```toml
name        = "Weather Wall"
description = "Wallpapers that react to local weather"
version     = 1
fillMode    = "fill"

[weather."*".weather]
clear   = "sunny.jpg"
cloudy  = "cloudy.jpg"
rainy   = "rain.jpg"
snowy   = "snow.jpg"
stormy  = "storm.jpg"
```

> **Note:** `lat` and `lon` are **not** set in the manifest — the user supplies their own coordinates in their personal `config.toml`. The theme only provides the image mapping.

---

## Fill Mode

Themes can suggest a `fillMode` in `manifest.toml`. However, if the user has set `fillMode` in their own config, **the user's value always wins** — it is preserved during theme merges.

| Value   | Effect |
|---------|--------|
| `fill`  | Cover output, centre-crop overflow *(default)* |
| `crop`  | Alias for `fill` |
| `fit`   | Fit inside output, letterbox with black |
| `scale` | Stretch to exact dimensions |
| `tile`  | Repeat at original size |

---

## Creating a Theme

### 1. Scaffold

```bash
wallman theme create ./my-new-theme
```

This creates the directory layout and a starter `manifest.toml`.

### 2. Add Images

Copy your wallpaper files into `my-new-theme/images/`.

### 3. Edit the Manifest

```bash
$EDITOR my-new-theme/manifest.toml
```

Set up the triggers you want (static, time-based, or weather-based).

---

## Packaging and Distribution

### Pack

```bash
wallman theme pack ./my-new-theme
# → my-new-theme.wallman
```

Wallman uses **Zstd-compressed tar archives** (`.wallman`) as the distribution format.

### Inspect

```bash
wallman pack inspect my-new-theme.wallman
```

Lists every file in the archive and its size.

### Install

```bash
wallman theme install my-new-theme.wallman
```

### Activate

```bash
wallman theme set my-new-theme
wallman daemon restart
```

---

## Output Wildcards

Using `"*"` as the output key in your manifest makes the theme work on any monitor configuration the user has — single screen, dual screen, ultrawide. Wallman's `OutputResolver` expands the wildcard to every detected output at runtime.

You can combine specific outputs with a wildcard fallback:

```toml
[background."DP-1"]
image = "primary.jpg"

[background."*"]
image = "secondary.jpg"
```

---

## Technical Details

| Detail | Value |
|--------|-------|
| File format | `tar` + `zstd` compression |
| Extension | `.wallman` |
| Image support | Any format supported by the `image` crate (JPG, PNG, WebP, BMP, GIF, …) |
| Rendering | Native `zwlr_layer_shell_v1` — no `swaybg` needed |
| Compatible compositors | Sway, Hyprland, river, labwc, and all wlr-layer-shell compositors |
