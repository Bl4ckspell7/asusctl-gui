#!/usr/bin/env bash
# Release version helper for asusctl-gui.
#
#   bump.sh compute <auto|patch|minor|major>
#       Print the next version (no leading "v"). Read-only.
#
#   bump.sh apply <version>
#       Bump the version in Cargo.toml, Cargo.lock and the AppStream metainfo,
#       (re)generate CHANGELOG.md and write this release's notes to
#       $RELEASE_NOTES (default /tmp/release-notes.md). Mutates the tree.
#
# "auto" derives the bump from Conventional Commits via git-cliff. The very
# first release (no tags yet) uses the current Cargo.toml version as-is.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

CARGO_TOML="Cargo.toml"
CARGO_LOCK="Cargo.lock"
METAINFO="data/com.github.bl4ckspell7.asusctl-gui.metainfo.xml"
RELEASE_NOTES="${RELEASE_NOTES:-/tmp/release-notes.md}"
REPO_URL="https://github.com/Bl4ckspell7/asusctl-gui"

# Current version: first `version = "x"` inside the [package] table.
current_version() {
    sed -nE '/^\[package\]/,/^\[/{s/^version = "([^"]+)".*/\1/p}' "$CARGO_TOML" | head -n1
}

# Highest existing release tag, without the leading "v".
latest_tag_version() {
    git tag --list 'v[0-9]*' --sort=-v:refname | head -n1 | sed 's/^v//'
}

# bump_semver <x.y.z> <major|minor|patch>
bump_semver() {
    local IFS=. ma mi pa
    read -r ma mi pa <<<"$1"
    case "$2" in
        major) echo "$((ma + 1)).0.0" ;;
        minor) echo "${ma}.$((mi + 1)).0" ;;
        patch) echo "${ma}.${mi}.$((pa + 1))" ;;
        *)
            echo "unknown bump level: $2" >&2
            exit 1
            ;;
    esac
}

compute() {
    local level="${1:-auto}" base next
    # First release: no tags yet -> keep the current Cargo.toml version.
    if [ -z "$(git tag --list 'v[0-9]*')" ]; then
        current_version
        return
    fi
    base="$(latest_tag_version)"
    [ -n "$base" ] || base="$(current_version)"
    case "$level" in
        auto)
            next="$(git cliff --bumped-version 2>/dev/null | sed 's/^v//')" || true
            # git-cliff returns the current version when nothing is bumpable.
            if [ -z "$next" ] || [ "$next" = "$base" ]; then
                next="$(bump_semver "$base" patch)"
            fi
            echo "$next"
            ;;
        *)
            bump_semver "$base" "$level"
            ;;
    esac
}

# Insert a new <release> as the first child of <releases>.
insert_metainfo_release() {
    local v="$1" d="$2" block
    block=$(
        cat <<EOF
        <release version="$v" date="$d">
            <url type="details">$REPO_URL/releases/tag/v$v</url>
            <description>
                <p>See the release notes.</p>
            </description>
        </release>
EOF
    )
    awk -v ins="$block" '
    { print }
    /<releases>/ && !done { print ins; done = 1 }
  ' "$METAINFO" >"$METAINFO.tmp"
    mv "$METAINFO.tmp" "$METAINFO"
}

apply() {
    local v="${1:?version required}" today
    today="$(date +%F)"

    # Cargo.toml: the version inside [package].
    sed -i -E '/^\[package\]/,/^\[/{s/^version = "[^"]+"/version = "'"$v"'"/}' "$CARGO_TOML"

    # Cargo.lock: the asusctl-gui package entry (version is the line after name).
    sed -i '/^name = "asusctl-gui"$/{n;s/^version = "[^"]*"/version = "'"$v"'"/}' "$CARGO_LOCK"

    # AppStream metainfo release entry.
    insert_metainfo_release "$v" "$today"

    # Changelog + this release's notes (Conventional Commits via git-cliff).
    git cliff --tag "v$v" -o CHANGELOG.md
    git cliff --tag "v$v" --unreleased --strip all >"$RELEASE_NOTES"
}

case "${1:-}" in
    compute) compute "${2:-auto}" ;;
    apply) apply "${2:-}" ;;
    *)
        echo "usage: $0 {compute <auto|patch|minor|major>|apply <version>}" >&2
        exit 1
        ;;
esac
