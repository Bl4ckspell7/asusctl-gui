//! DMI/SMBIOS information from `/sys/class/dmi/id`.
//!
//! Most fields there are world-readable, so they are read directly even inside
//! the Flatpak sandbox (`/sys` is bind-mounted read-only). The serial number is
//! the exception: `product_serial` is mode 0400 root, so it is read on demand
//! through `pkexec`, which triggers a polkit prompt.

use std::fs;
use std::sync::OnceLock;

use super::dbus::host_command;
use super::error::{AsusctlError, Result};
use super::types::DmiInfo;

const DMI_DIR: &str = "/sys/class/dmi/id";
const DMI_SERIAL_PATH: &str = "/sys/class/dmi/id/product_serial";

/// Placeholder values OEMs leave in the SMBIOS tables. Compared case-insensitively.
const DMI_PLACEHOLDERS: &[&str] = &[
    "default string",
    "to be filled by o.e.m.",
    "system serial number",
    "not specified",
    "not applicable",
    "none",
    "unknown",
    "n/a",
];

static DMI_INFO: OnceLock<DmiInfo> = OnceLock::new();

// ============================================================================
// Public API
// ============================================================================

/// Read the world-readable DMI fields once and cache them for the process lifetime.
///
/// Fields that are missing or contain OEM placeholder text come back empty; the
/// UI decides how to present that.
pub fn get_dmi_info() -> &'static DmiInfo {
    DMI_INFO.get_or_init(|| DmiInfo {
        bios_version: read_dmi_field("bios_version"),
        bios_date: read_dmi_field("bios_date"),
        bios_vendor: read_dmi_field("bios_vendor"),
        board_vendor: read_dmi_field("board_vendor"),
    })
}

/// Read the machine serial number via `pkexec cat` (polkit prompt).
///
/// Blocks until the user answers the polkit dialog, so callers must run this
/// off the main loop.
///
/// `Ok(None)` means the read succeeded but the firmware has no real serial
/// (placeholder text) — retrying will not help. `Err` means the read itself
/// failed, e.g. because authorization was refused.
pub fn read_serial_number() -> Result<Option<String>> {
    let output = host_command("pkexec")
        .args(["cat", DMI_SERIAL_PATH])
        .output()
        .map_err(|e| AsusctlError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(interpret_pkexec_failure(output.status.code()));
    }

    let serial = sanitize_dmi_value(&String::from_utf8_lossy(&output.stdout));
    if serial.is_empty() {
        return Ok(None);
    }
    Ok(Some(serial))
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// Read a single sysfs DMI attribute, returning an empty string when it is
/// unreadable or holds a placeholder value.
fn read_dmi_field(name: &str) -> String {
    fs::read_to_string(format!("{DMI_DIR}/{name}"))
        .map(|value| sanitize_dmi_value(&value))
        .unwrap_or_default()
}

/// Trim a raw DMI value and blank out well-known OEM placeholder text.
fn sanitize_dmi_value(raw: &str) -> String {
    let trimmed = raw.trim();
    let lowered = trimmed.to_lowercase();
    if DMI_PLACEHOLDERS.contains(&lowered.as_str()) {
        return String::new();
    }
    trimmed.to_string()
}

/// Turn a `pkexec` exit code into an actionable error.
///
/// pkexec reserves 126 for "authorization could not be obtained" (the dialog
/// was dismissed or the password was wrong) and 127 for "pkexec/the program
/// could not be found". Anything else came from `cat` itself.
fn interpret_pkexec_failure(code: Option<i32>) -> AsusctlError {
    match code {
        Some(126) => AsusctlError::CommandFailed("Authorization was not granted".to_string()),
        Some(127) => AsusctlError::CommandFailed("pkexec is not available".to_string()),
        Some(code) => AsusctlError::CommandFailed(format!("pkexec cat exited with code {code}")),
        None => AsusctlError::CommandFailed("pkexec was terminated by a signal".to_string()),
    }
}

/// Convert an SMBIOS BIOS release date (`MM/DD/YYYY`) to ISO 8601 `YYYY-MM-DD`.
///
/// Returns `None` for anything that does not match that shape, so callers can
/// fall back to the raw firmware string.
fn to_iso_date(raw: &str) -> Option<String> {
    let mut parts = raw.split('/');
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    let year: u32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// Format the BIOS version and release date for display, degrading gracefully
/// when either half is missing. The date is normalised to ISO 8601.
pub fn format_bios(info: &DmiInfo) -> String {
    let date = to_iso_date(&info.bios_date).unwrap_or_else(|| info.bios_date.clone());

    match (info.bios_version.as_str(), date.as_str()) {
        ("", "") => "Unknown".to_string(),
        (version, "") => version.to_string(),
        ("", date) => date.to_string(),
        (version, date) => format!("{version} · {date}"),
    }
}

#[cfg(test)]
#[path = "tests/dmi_tests.rs"]
mod tests;
