# asusctl-gui

[![Rust](https://github.com/Bl4ckspell7/asusctl-gui/actions/workflows/rust.yml/badge.svg)](https://github.com/Bl4ckspell7/asusctl-gui/actions/workflows/rust.yml)
[![Flatpak](https://github.com/Bl4ckspell7/asusctl-gui/actions/workflows/flatpak.yml/badge.svg)](https://github.com/Bl4ckspell7/asusctl-gui/actions/workflows/flatpak.yml)

A GTK4/Libadwaita GUI for [asusctl](https://gitlab.com/asus-linux/asusctl) - manage your ASUS ROG laptop settings.

## Features

- **About** - View laptop info, driver status, and supported features
- **Aura** - Manage keyboard lighting modes and colors
- **Power** - Set power profiles for AC/battery
- **Slash** - Control slash lighting on the back of the display

## Screenshots

|                                           About                                           |                                           Aura                                           |
| :---------------------------------------------------------------------------------------: | :--------------------------------------------------------------------------------------: |
| ![About](https://github.com/user-attachments/assets/faa673ac-7539-4d0b-b5ad-639d926b79b4) | ![Aura](https://github.com/user-attachments/assets/215c05e5-ad5c-4bcd-8a28-131b55112fd0) |

|                                           Power                                           |                                           Slash                                           |
| :---------------------------------------------------------------------------------------: | :---------------------------------------------------------------------------------------: |
| ![Power](https://github.com/user-attachments/assets/a07cf6d4-880f-4b10-9266-6b24000129bc) | ![Slash](https://github.com/user-attachments/assets/f9cd5487-a12b-4c5f-8ae9-9732e3b0c3da) |

## Requirements

- `asusctl` installed and configured

## Installation

Download the latest `asusctl-gui-x86_64.flatpak` artifact from [GitHub Actions](https://github.com/Bl4ckspell7/asusctl-gui/actions/workflows/flatpak.yml), then install:

```bash
flatpak install --user ./asusctl-gui.flatpak
flatpak run com.github.bl4ckspell7.asusctl-gui
```

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for building from source, testing, and coverage instructions.

## AI Disclaimer

Portions of this code have been generated with assistance from AI tools. All AI-generated code is reviewed by the maintainer with the same standards applied to any other contribution.

## License

GPL-3.0
