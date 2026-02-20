//! Slash LED bar control.

use super::dbus::{
    SLASH_INTERFACE, call_dbus_method, get_slash_path, parse_dbus_bool, parse_dbus_byte,
    read_dbus_property_at, run_asusctl,
};
use super::error::{AsusctlError, Result};
use super::types::SlashMode;

// ============================================================================
// Public API - Enable/Disable
// ============================================================================

/// Enable slash LED bar.
pub fn enable_slash() -> Result<()> {
    run_asusctl(&["slash", "--enable"])?;
    log::info!("Enabled slash LED bar");
    Ok(())
}

/// Disable slash LED bar.
pub fn disable_slash() -> Result<()> {
    run_asusctl(&["slash", "--disable"])?;
    log::info!("Disabled slash LED bar");
    Ok(())
}

// ============================================================================
// Public API - Settings
// ============================================================================

/// Set slash brightness (0-255).
pub fn set_slash_brightness(brightness: u8) -> Result<()> {
    run_asusctl(&["slash", "--brightness", &brightness.to_string()])?;
    log::info!("Set slash brightness to {brightness}");
    Ok(())
}

/// Set slash mode.
pub fn set_slash_mode(mode: SlashMode) -> Result<()> {
    run_asusctl(&["slash", "--mode", &mode.to_string()])?;
    log::info!("Set slash mode to {mode}");
    Ok(())
}

/// Set slash interval (0-5).
pub fn set_slash_interval(interval: u8) -> Result<()> {
    run_asusctl(&["slash", "--interval", &interval.to_string()])?;
    log::info!("Set slash interval to {interval}");
    Ok(())
}

// ============================================================================
// Public API - State Getters (D-Bus)
// ============================================================================

/// Get slash enabled state via D-Bus.
pub fn get_slash_enabled() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "Enabled")?;
    parse_dbus_bool(&output)
}

/// Get slash brightness via D-Bus.
pub fn get_slash_brightness() -> Result<u8> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "Brightness")?;
    parse_dbus_byte(&output)
}

/// Get slash interval via D-Bus.
pub fn get_slash_interval() -> Result<u8> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "Interval")?;
    parse_dbus_byte(&output)
}

/// Get slash mode via D-Bus DeviceState method.
///
/// The `Mode` property doesn't reflect changes made via asusctl,
/// so we use the `DeviceState` method which returns `(byyu)`:
/// (enabled, brightness, interval, mode).
pub fn get_slash_mode() -> Result<SlashMode> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = call_dbus_method(path, SLASH_INTERFACE, "DeviceState")?;

    // Output format: "byyu true 255 0 6"
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(AsusctlError::ParseError(format!(
            "Invalid DeviceState format: {output}"
        )));
    }

    let mode_val: u32 = parts[4]
        .parse()
        .map_err(|_| AsusctlError::ParseError(format!("Invalid mode value: {}", parts[4])))?;

    SlashMode::from_dbus_value(mode_val as u8)
        .ok_or_else(|| AsusctlError::ParseError(format!("Unknown slash mode value: {mode_val}")))
}

// ============================================================================
// Public API - Show-On Event Getters (D-Bus only)
// ============================================================================

pub fn get_slash_show_on_boot() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "ShowOnBoot")?;
    parse_dbus_bool(&output)
}

pub fn get_slash_show_on_shutdown() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "ShowOnShutdown")?;
    parse_dbus_bool(&output)
}

pub fn get_slash_show_on_sleep() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "ShowOnSleep")?;
    parse_dbus_bool(&output)
}

pub fn get_slash_show_on_battery() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "ShowOnBattery")?;
    parse_dbus_bool(&output)
}

pub fn get_slash_show_battery_warning() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "ShowBatteryWarning")?;
    parse_dbus_bool(&output)
}

// ============================================================================
// Public API - Show-On Event Setters
// ============================================================================

pub fn set_slash_show_on_boot(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-boot",
        if value { "true" } else { "false" },
    ])?;
    log::info!("Set slash show-on-boot to {value}");
    Ok(())
}

pub fn set_slash_show_on_shutdown(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-shutdown",
        if value { "true" } else { "false" },
    ])?;
    log::info!("Set slash show-on-shutdown to {value}");
    Ok(())
}

pub fn set_slash_show_on_sleep(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-sleep",
        if value { "true" } else { "false" },
    ])?;
    log::info!("Set slash show-on-sleep to {value}");
    Ok(())
}

pub fn set_slash_show_on_battery(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-battery",
        if value { "true" } else { "false" },
    ])?;
    log::info!("Set slash show-on-battery to {value}");
    Ok(())
}

pub fn set_slash_show_battery_warning(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-battery-warning",
        if value { "true" } else { "false" },
    ])?;
    log::info!("Set slash show-battery-warning to {value}");
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_mode_from_dbus_value() {
        assert_eq!(SlashMode::from_dbus_value(0), Some(SlashMode::Static));
        assert_eq!(SlashMode::from_dbus_value(6), Some(SlashMode::Flow));
        assert_eq!(SlashMode::from_dbus_value(15), Some(SlashMode::Buzzer));
        assert_eq!(SlashMode::from_dbus_value(16), None);
        assert_eq!(SlashMode::from_dbus_value(255), None);
    }
}
