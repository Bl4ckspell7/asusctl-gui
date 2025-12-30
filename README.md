# asusctl-gui

A GTK4/Libadwaita GUI for [asusctl](https://gitlab.com/asus-linux/asusctl) - manage your ASUS ROG laptop settings.

## Features

- **About** - View laptop info, driver status, and supported features
- **Aura** - Manage keyboard lighting modes and colors
- **Profile** - Set power profiles for AC/battery
- **Slash** - Control slash lighting on the back of the display

## Requirements

- GTK4 4.20+
- libadwaita 1.8+
- asusctl installed and configured
- Rust 1.83+

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```
