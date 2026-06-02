# Development

## Prerequisites

- GTK4 4.22+
- libadwaita 1.9+
- Rust 1.85+ (edition 2024)
- asusctl installed and configured

## Cargo

### Setup

Copy the GSettings schema to your local schemas directory and compile:

```bash
cp resources/com.github.bl4ckspell7.asusctl-gui.gschema.xml ~/.local/share/glib-2.0/schemas/

glib-compile-schemas ~/.local/share/glib-2.0/schemas/
```

### Update dependencies

```bash
cargo update
```

### Upgrade dependencies

```bash
cargo upgrade -i
```

### Run

```bash
cargo run
```

### Build

```bash
cargo build
```

## Flatpak

### Prerequisites

Install Flatpak, flatpak-builder, and the GNOME SDK (Arch Linux):

```bash
sudo pacman -S flatpak flatpak-builder
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
```

### Generate cargo sources

The Flatpak build runs offline, so all crate sources must be vendored into `cargo-sources.json`. Use [flatpak-cargo-generator](https://github.com/niclas-nickel/flatpak-cargo-generator):

```bash
uv tool install flatpak-cargo-generator
```

```bash
flatpak-cargo-generator Cargo.lock -o cargo-sources.json
```

Re-run this whenever `Cargo.lock` changes.

### Build

```bash
flatpak-builder --user --install --force-clean builddir com.github.bl4ckspell7.asusctl-gui.yml
```

### Run

```bash
flatpak run com.github.bl4ckspell7.asusctl-gui
```

### How it works

The app runs inside a Flatpak sandbox but needs access to host-side tools (`asusctl`, `busctl`, `powerprofilesctl`). This is handled by:

- **`flatpak-spawn --host`** — all host commands are automatically wrapped when the app detects it is running inside Flatpak (via `/.flatpak-info`). See `host_command()` in `src/backend/dbus.rs`.
- **`--talk-name=org.freedesktop.Flatpak`** — grants permission for `flatpak-spawn --host` to work.
- **`--system-talk-name=xyz.ljones.Asusd`** — grants direct D-Bus access to the asusd system bus.

## Testing

```bash
cargo test
```

### Coverage

Install cargo-llvm-cov:

```bash
cargo install cargo-llvm-cov
```

Generate HTML coverage report:

```bash
cargo llvm-cov --html
```

The report will be generated in `target/llvm-cov/html/index.html`.

Generate text summary:

```bash
cargo llvm-cov
```

## Releasing

Releases are cut by the **Release** workflow (`.github/workflows/release.yml`) — triggered manually, never on push. Commit messages must follow [Conventional Commits](https://www.conventionalcommits.org); the version bump and changelog are derived from them.

1. **Actions → Release → Run workflow**, and pick the bump:
   - `auto` (default) — derived from commits since the last tag (`feat` → minor, `fix` → patch, breaking → major).
   - `patch` / `minor` / `major` — force a level.

   Leave **`dry_run` checked** the first time.

2. **Review the dry-run job summary** — it prints the computed version and release notes without changing anything.
3. **Re-run with `dry_run` unchecked.** The workflow bumps the version (`Cargo.toml`, `Cargo.lock`, metainfo), updates `CHANGELOG.md`, builds the Flatpak bundle, commits + tags `vX.Y.Z`, and opens a **draft** GitHub Release with the bundle + `SHA256SUMS.txt`.
4. **Review the draft, then publish it.** Publishing locks the tag and assets ([immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)) and generates a build attestation.

### Preview locally (optional)

```bash
uv tool install git-cliff
```

```bash
git cliff --bumped-version   # next version
git cliff --unreleased       # notes for the upcoming release
```
