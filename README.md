# asusctl-gui

A GTK4/Libadwaita GUI for [asusctl](https://gitlab.com/asus-linux/asusctl) - manage your ASUS ROG laptop settings.

## Features

- **About** - View laptop info, driver status, and supported features
- **Aura** - Manage keyboard lighting modes and colors
- **Power** - Set power profiles for AC/battery
- **Slash** - Control slash lighting on the back of the display

## Screenshots

|                                           About                                           |                                           Aura                                           |
| :---------------------------------------------------------------------------------------: | :--------------------------------------------------------------------------------------: |
| ![About](https://github.com/user-attachments/assets/d9398083-b044-4c1d-9c9f-986a9bd8178d) | ![Aura](https://github.com/user-attachments/assets/1ef975c1-a2f9-4748-9355-1e15fa52e6d0) |

|                                           Power                                           |                                           Slash                                           |
| :---------------------------------------------------------------------------------------: | :---------------------------------------------------------------------------------------: |
| ![Power](https://github.com/user-attachments/assets/599b8f97-843f-4fa1-b585-619227fc5c75) | ![Slash](https://github.com/user-attachments/assets/f7665dfc-2ac5-44a6-8e48-9eeca01b230e) |

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
