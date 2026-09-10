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
    run_slash_set(&["--enable"])?;
    log::info!("Enabled slash LED bar");
    Ok(())
}

/// Disable slash LED bar.
pub fn disable_slash() -> Result<()> {
    run_slash_set(&["--disable"])?;
    log::info!("Disabled slash LED bar");
    Ok(())
}

// ============================================================================
// Public API - Settings
// ============================================================================

/// Set slash brightness (0-255).
pub fn set_slash_brightness(brightness: u8) -> Result<()> {
    run_slash_set(&["--brightness", &brightness.to_string()])?;
    log::info!("Set slash brightness to {brightness}");
    Ok(())
}

/// Set slash mode.
pub fn set_slash_mode(mode: SlashMode) -> Result<()> {
    run_slash_set(&["--mode", &mode.to_string()])?;
    log::info!("Set slash mode to {mode}");
    Ok(())
}

/// Set slash interval (0-5).
pub fn set_slash_interval(interval: u8) -> Result<()> {
    run_slash_set(&["--interval", &interval.to_string()])?;
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
/// `DeviceState` returns `(byyu)` (enabled, brightness, interval, mode), with
/// mode represented by the `SlashMode` ordinal from 0 through 15. In asusctl
/// 6.4.0, the standalone `Mode` property instead exposes the raw hardware mode
/// byte (for example, `6` represents `Static`), so it does not use this mapping.
pub fn get_slash_mode() -> Result<SlashMode> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = call_dbus_method(path, SLASH_INTERFACE, "DeviceState")?;

    // Output format: "byyu true 255 0 0"
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
    run_slash_set(&["--show-on-boot", bool_arg(value)])?;
    log::info!("Set slash show-on-boot to {value}");
    Ok(())
}

pub fn set_slash_show_on_shutdown(value: bool) -> Result<()> {
    run_slash_set(&["--show-on-shutdown", bool_arg(value)])?;
    log::info!("Set slash show-on-shutdown to {value}");
    Ok(())
}

pub fn set_slash_show_on_sleep(value: bool) -> Result<()> {
    run_slash_set(&["--show-on-sleep", bool_arg(value)])?;
    log::info!("Set slash show-on-sleep to {value}");
    Ok(())
}

pub fn set_slash_show_on_battery(value: bool) -> Result<()> {
    run_slash_set(&["--show-on-battery", bool_arg(value)])?;
    log::info!("Set slash show-on-battery to {value}");
    Ok(())
}

pub fn set_slash_show_battery_warning(value: bool) -> Result<()> {
    run_slash_set(&["--show-battery-warning", bool_arg(value)])?;
    log::info!("Set slash show-battery-warning to {value}");
    Ok(())
}

// ============================================================================
// Private Helpers
// ============================================================================

fn slash_set_args<'a>(args: &'a [&'a str]) -> Vec<&'a str> {
    let mut command = Vec::with_capacity(args.len() + 2);
    command.extend_from_slice(&["slash", "set"]);
    command.extend_from_slice(args);
    command
}

fn run_slash_set(args: &[&str]) -> Result<()> {
    run_asusctl(&slash_set_args(args))?;
    Ok(())
}

fn bool_arg(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[cfg(test)]
#[path = "tests/slash_tests.rs"]
mod tests;
