# Barricade Sound [![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

<div align="center" style="margin-bottom: 20px;">

[English](README_EN.md) | [Русский](README_RU.md)

</div>

Sound control popup for barricade-bar, built with Rust, GTK 3 and Layer Shell (Wayland).

## Description

Barricade Sound is a lightweight popup window that appears in the top-right corner of the screen and lets you control audio without leaving the bar. It uses:
- **GTK 3** and **gtk-layer-shell** to draw an overlay window on Wayland
- **wpctl** (WirePlumber / PipeWire) for volume, mute and device control
- **playerctl** (MPRIS) for media player control
- **Rust** for performance and memory safety

## Features

- Output volume, mute and default output device selection
- Microphone gain, mute and default input device selection
- Media player controls: previous / play-pause / next
- Seek bar with elapsed and total time
- Album cover (local files and remote URLs with caching)
- Custom styling through a CSS file
- Closes on `Esc` or when the window loses focus

## Installation

### Requirements

- Rust 1.85+ (edition 2024)
- GTK 3.x
- GTK Layer Shell 0.7+
- A Wayland compositor with `wlr-layer-shell` support
- Runtime tools: `wpctl` (WirePlumber), `playerctl`, `curl`

### Nix Users

This project includes a `shell.nix` for reproducible development environment:

```bash
nix-shell
```

The shell provides all dependencies: GTK 3, cairo, pango, gdk-pixbuf, gtk-layer-shell, playerctl and build tools.

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run --release
```

## Usage

Run the binary (for example, bind it to a button in barricade-bar):

```bash
./target/release/Barricade-sound
```

The window can be closed with `Esc` or by clicking outside of it.

### Configuration

The config file is located at `~/.config/barricade/bar.bst` (shared with barricade-bar) and is created on first run. To apply custom styles, set the path to a CSS file:

```yaml
stylesheet: "~/.config/barricade/style.css"
```

Relative paths are resolved from `~/.config/barricade/`. An empty value means no custom styles.

## Project Structure

```
src/
├── main.rs              # Entry point, layer-shell window, CSS loading
├── ui.rs                # Window layout and periodic widget updates
├── lib/
│   ├── mod.rs           # Library modules
│   ├── audio.rs         # Volume / mic / devices via wpctl
│   ├── player.rs        # Media player control via playerctl
│   ├── widget.rs        # Common widget trait
│   └── parcer.rs        # Configuration file parser
└── widgets/
    ├── mod.rs           # Widget modules
    ├── sink_control.rs  # Output volume and device list
    ├── mic_cintrol.rs   # Microphone gain and device list
    └── player.rs        # Player controls, cover and seek bar
```

## Development

### Build and run in development mode

```bash
cargo run
```

### Linting and type checking

```bash
cargo clippy
cargo check
```

## License

Apache License 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
