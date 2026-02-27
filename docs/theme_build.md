# building Wallman Themes

A Wallman theme is a portable package that contains wallpapers and a configuration manifest explaining how those images should be applied.

## Theme Structure

A theme directory should look like this:

```text
my-theme/
├── manifest.toml
└── images/
    ├── day.jpg
    ├── night.jpg
    └── background.png
```

### The `manifest.toml`

The manifest is a standard Wallman configuration file. When a theme is installed, these settings are used to drive the daemon.

```toml
name = "Moon Wall"
description = "A sleek cosmic theme"
version = 1

[background."*"]
image = "background.png"
fill_mode = "fill"

[timeConfig."*"]
day = "day.jpg"
night = "night.jpg"
day_range = "07-20"
```

> **Note**: Only set the name of the image file, not the full path. The images are stored in the `images/` folder of the theme directory.

---

## Creating a Theme

### 1. Scaffold

Use the CLI to create the basic structure:

```bash
wallman theme create my-new-theme
```

### 2. Add Assets

Move your images into the `images/` folder of the new directory.

### 3. Configure

Edit the `manifest.toml` to define your triggers (Static, DayTime, or Weather).

---

## Packaging and Distribution

### Packing

To distribute your theme, you must pack it into a `.wallman` file. Wallman uses Zstd compression and Tar archiving internally.

```bash
wallman theme pack ./my-new-theme
```

This generates `my-new-theme.wallman`.

### Installing

To share with others, they can simply run:

```bash
wallman theme install my-new-theme.wallman
```

### Activation

After installation, the theme must be activated to take effect:

```bash
wallman theme set my-new-theme
```

---

## Technical Details

- **File Format**: `.wallman` files are `tar.zst` archives.
- **Image Support**: Wallman supports any image format that `swaybg` can handle (JPG, PNG, etc.).
- **Output Wildcards**: By using `"*"` as the output key in your manifest, your theme will automatically work on any number of monitors the user has connected.
