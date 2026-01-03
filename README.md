# asusctl-gui

A GTK4/Libadwaita GUI for [asusctl](https://gitlab.com/asus-linux/asusctl) - manage your ASUS ROG laptop settings.

## Features

- **About** - View laptop info, driver status, and supported features
- **Aura** - Manage keyboard lighting modes and colors
- **Power** - Set power profiles for AC/battery
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

## Acknowledgements

This project was developed with assistance from Claude AI.

## License

GPL-3.0

## Develop

```
cp resources/com.github.bl4ckspell7.asusctl-gui.gschema.xml ~/.local/share/glib-2.0/schemas/

glib-compile-schemas ~/.local/share/glib-2.0/schemas/
```
