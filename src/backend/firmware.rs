//! Firmware settings from the asus-armoury driver that apply immediately.
//!
//! asusd writes these straight to the firmware and saves them in its config,
//! so they persist across reboots.

use super::dbus::{
    ARMOURY_INTERFACE, armoury_attr_path, parse_dbus_int, read_dbus_property_at, run_asusctl,
};
use super::error::{AsusctlError, Result};

const BOOT_SOUND_ATTR: &str = "boot_sound";

// ============================================================================
// Public API - Boot Sound
// ============================================================================

/// Get whether the POST boot sound is on via D-Bus.
pub fn get_boot_sound() -> Result<bool> {
    let output = read_dbus_property_at(
        &armoury_attr_path(BOOT_SOUND_ATTR),
        ARMOURY_INTERFACE,
        "CurrentValue",
    )?;
    boot_sound_from_value(parse_dbus_int(&output)?)
}

/// Turn the POST boot sound on or off. It is heard at the next boot.
pub fn set_boot_sound(enabled: bool) -> Result<()> {
    let value = if enabled { "1" } else { "0" };
    run_asusctl(&["armoury", "set", BOOT_SOUND_ATTR, value])?;
    log::info!("Turned boot sound {}", if enabled { "on" } else { "off" });
    Ok(())
}

// ============================================================================
// Private Helpers
// ============================================================================

fn boot_sound_from_value(value: i32) -> Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown boot sound value: {value}"
        ))),
    }
}

#[cfg(test)]
#[path = "tests/firmware_tests.rs"]
mod tests;
