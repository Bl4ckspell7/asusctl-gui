//! GNOME custom keyboard shortcut that launches the app.
//!
//! The shortcut is a regular GNOME custom keybinding (the same kind of entry
//! Settings → Keyboard → Custom Shortcuts creates), so it also works while the
//! app is closed. It is managed through the host's `gsettings`, because the
//! Flatpak runtime neither ships the settings-daemon schemas nor writes to the
//! host's dconf database.

use gtk4::glib::prelude::ToVariant;
use gtk4::glib::{self, VariantTy};

use super::dbus::{host_command, is_flatpak};
use super::error::{AsusctlError, Result};
use crate::app::{APP_ID, APP_NAME};

const MEDIA_KEYS_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys";
const CUSTOM_KEYBINDINGS_KEY: &str = "custom-keybindings";
const CUSTOM_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
/// The app's own entry. GNOME accepts any path in the custom keybinding list,
/// and a fixed one keeps the app away from the user's other shortcuts.
const SHORTCUT_PATH: &str =
    "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/asusctl-gui/";
const BINARY_NAME: &str = env!("CARGO_BIN_NAME");
/// The ROG key (`XF86Launch1`) as GNOME stores it.
const ROG_KEY_BINDING: &str = "Launch1";

// ============================================================================
// Public API
// ============================================================================

/// Whether the session is GNOME, the only desktop whose shortcuts are managed here.
pub fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktops| {
        desktops
            .split(':')
            .any(|desktop| desktop.eq_ignore_ascii_case("GNOME"))
    })
}

/// Read the app's launch shortcut.
///
/// `Ok(None)` means there is no shortcut; `Ok(Some(""))` means the entry exists
/// but has no key assigned. Blocks on host `gsettings` calls, so callers must
/// run this off the main loop.
pub fn get_launch_shortcut() -> Result<Option<String>> {
    if !has_shortcut(&read_path_list()?) {
        return Ok(None);
    }

    let binding = parse_string(&run_gsettings(&["get", &custom_schema(), "binding"])?)?;
    Ok(Some(binding))
}

/// Add a custom shortcut that launches the app with the ROG key.
///
/// Does nothing if the app's shortcut already exists.
pub fn add_launch_shortcut() -> Result<()> {
    let mut paths = read_path_list()?;
    if has_shortcut(&paths) {
        return Ok(());
    }

    // Start from the schema defaults in case an old entry left keys behind
    let schema = custom_schema();
    run_gsettings(&["reset-recursively", &schema])?;
    run_gsettings(&["set", &schema, "name", &quote(APP_NAME)])?;
    run_gsettings(&["set", &schema, "command", &quote(&launch_command())])?;
    run_gsettings(&["set", &schema, "binding", &quote(ROG_KEY_BINDING)])?;

    // Register the entry last so GNOME never picks up a half-written shortcut
    paths.push(SHORTCUT_PATH.to_string());
    write_path_list(&paths)
}

/// Remove the app's launch shortcut, if there is one.
pub fn remove_launch_shortcut() -> Result<()> {
    let mut paths = read_path_list()?;
    if has_shortcut(&paths) {
        paths.retain(|path| path != SHORTCUT_PATH);
        write_path_list(&paths)?;
    }

    run_gsettings(&["reset-recursively", &custom_schema()])?;
    Ok(())
}

// ============================================================================
// gsettings Access
// ============================================================================

/// Run host `gsettings` and return its trimmed stdout.
fn run_gsettings(args: &[&str]) -> Result<String> {
    let output = host_command("gsettings")
        .args(args)
        .output()
        .map_err(|e| AsusctlError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("gsettings {} failed", args.join(" "))
        } else {
            stderr
        };
        return Err(AsusctlError::CommandFailed(message));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn read_path_list() -> Result<Vec<String>> {
    parse_path_list(&run_gsettings(&[
        "get",
        MEDIA_KEYS_SCHEMA,
        CUSTOM_KEYBINDINGS_KEY,
    ])?)
}

fn write_path_list(paths: &[String]) -> Result<()> {
    run_gsettings(&[
        "set",
        MEDIA_KEYS_SCHEMA,
        CUSTOM_KEYBINDINGS_KEY,
        &format_path_list(paths),
    ])?;
    Ok(())
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// `schema:path` argument for the app's custom keybinding entry.
fn custom_schema() -> String {
    format!("{CUSTOM_SCHEMA}:{SHORTCUT_PATH}")
}

/// Whether the custom keybinding list contains the app's entry.
fn has_shortcut(paths: &[String]) -> bool {
    paths.iter().any(|path| path == SHORTCUT_PATH)
}

/// Parse `gsettings get` output of an `as` key (e.g. `['/a/', '/b/']` or `@as []`).
fn parse_path_list(text: &str) -> Result<Vec<String>> {
    glib::Variant::parse(Some(VariantTy::STRING_ARRAY), text)
        .ok()
        .and_then(|variant| variant.get::<Vec<String>>())
        .ok_or_else(|| AsusctlError::ParseError(format!("invalid string list: {text}")))
}

/// Parse `gsettings get` output of an `s` key (e.g. `'Launch1'`).
fn parse_string(text: &str) -> Result<String> {
    glib::Variant::parse(Some(VariantTy::STRING), text)
        .ok()
        .and_then(|variant| variant.get::<String>())
        .ok_or_else(|| AsusctlError::ParseError(format!("invalid string: {text}")))
}

/// Format a path list as GVariant text for `gsettings set`.
fn format_path_list(paths: &[String]) -> String {
    paths.to_variant().print(false).to_string()
}

/// Format a string as GVariant text for `gsettings set`.
fn quote(value: &str) -> String {
    value.to_variant().print(false).to_string()
}

/// Command line the shortcut runs to launch the app.
fn launch_command() -> String {
    if is_flatpak() {
        return format!("flatpak run {APP_ID}");
    }

    match std::env::current_exe() {
        Ok(exe) => glib::shell_quote(exe).to_string_lossy().into_owned(),
        Err(_) => BINARY_NAME.to_string(),
    }
}

#[cfg(test)]
#[path = "tests/shortcut_tests.rs"]
mod tests;
