# Changelog

All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-06-02

### Bug Fixes

- *(backend)* Detect missing asusctl under Flatpak
- *(flatpak)* Build on the stable branch

### Documentation

- *(readme)* Point install to GitHub releases
- Correct DEVELOPMENT.md prerequisite versions
## [0.1.0] - 2026-06-02

### Features

- Periodically refresh data
- Implement aura lighting mode controls
- Add keyboard shortcuts for Preferences/About, move Quit to separated menu section
- Log all backend setting changes to stderr
- Add custom rainbow effect with adjustable speed
- Add asus-armoury driver detection on About page
- Add distro and kernel version, reorganize About page sections
- Add Flatpak build support
- Add flatpak-spawn --host support for sandbox compatibility
- Update app icon to laptop with wrench design
- Async feature detection with loading spinner
- Replace eprintln! with log + env_logger
- Redesign app icon with gradient background
- Upgrade to libadwaita 1.9 and migrate to adw::Sidebar
- Collapse sidebar on narrow windows using adw::Breakpoint
- Upgrade to libadwaita 1.9 and migrate to adw::Sidebar
- Collapse sidebar on narrow windows using adw::Breakpoint
- *(ci)* Add manual release workflow

### Bug Fixes

- Gschema description
- About page: use different icons
- Update backend cli commands for asusctl version 6.3.0
- Prevent slash from auto-enabling on app startup
- Add refresh guards to power and aura pages
- Aura lighting mode not detected when changed externally
- Update CLI commands for asusctl 6.3.2
- Detect features at startup and gate UI for unsupported hardware
- Init pacman keyring before installing dependencies in CI
- Use freeze_notify guards to prevent GTK state warnings
- Read host os-release in Flatpak for real distro name
- Remove stack transition to prevent page switch flicker
- Use per-page ScrolledWindow to prevent scroll state transfer
- Regenerate `cargo-sources.json` for flatpak build, caused by `cargo.toml` changes
- Correct aura D-Bus mode mapping and color group visibility
- Use exact word matching for aura mode parsing to prevent substring false positives
- Regenerate cargo-sources.json for updated Cargo.lock
- Regenerate cargo-sources.json for updated Cargo.lock
- Display charge limit scale as integer (no decimals)
- Use v4_20 feature flag — v4_22 requires unreleased GTK 4.22
- Use v4_20 feature flag — v4_22 requires unreleased GTK 4.22
- Prevent content clipping on narrow window resize
- Increase minimum window height from 140 to 314
- Regenerate cargo-sources.json for updated Cargo.lock
- Refresh Cargo.lock and cargo-sources.json

### Refactor

- Split monolithic backend into feature modules
- Replace slash config file parsing with D-Bus reads
- Extract app name constant and update window title
- Extract default window dimensions as constants
- Use CARGO_PKG_VERSION for about dialog version
- Move unit tests into separate files under backend/tests/
- Replace aura mode toggle buttons with ComboRow dropdown
- Remove custom rainbow effect in favor of native asusctl rainbow modes

### Documentation

- Fix power page name
- Add screenshots section to README
- Add DEVELOPMENT.md and simplify README
- Reorganize README and DEVELOPMENT docs
- Clean up README and add flatpak install instructions
- Minor edit in `README.me`
- Minor edit in README.me
- Add CI build status badges to README
- Update AI disclaimer section in README
- Add trademark notice to README
- Add cargo update and cargo upgrade commands to DEVELOPMENT.md
- Add link to asusctl in requirements section

### Other

- Use correct website url
- Add sections Acknowledgements & License

