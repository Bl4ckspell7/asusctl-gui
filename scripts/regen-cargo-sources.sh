#!/usr/bin/env bash
set -euo pipefail

# Regenerate the Flatpak cargo sources from Cargo.lock.
# Run by Renovate as a postUpgradeTask after it updates Cargo.lock.
# flatpak-cargo-generator is a Python tool.
if ! command -v flatpak-cargo-generator >/dev/null 2>&1; then
  install-tool python 3.14.6
  python -m pip install --user flatpak-cargo-generator
  export PATH="$HOME/.local/bin:$PATH"
fi
flatpak-cargo-generator Cargo.lock -o cargo-sources.json
