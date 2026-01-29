//! Slash LED bar control.

use std::fs;
use std::str::FromStr;

use super::dbus::{
    get_slash_path, parse_dbus_bool, parse_dbus_byte, read_dbus_property_at, run_asusctl,
    SLASH_CONFIG_PATH, SLASH_INTERFACE,
};
use super::error::{AsusctlError, Result};
use super::types::{SlashMode, SlashState};

// ============================================================================
// Public API - Enable/Disable
// ============================================================================

/// Enable slash LED bar.
pub fn enable_slash() -> Result<()> {
    run_asusctl(&["slash", "--enable"])?;
    Ok(())
}

/// Disable slash LED bar.
pub fn disable_slash() -> Result<()> {
    run_asusctl(&["slash", "--disable"])?;
    Ok(())
}

// ============================================================================
// Public API - Settings
// ============================================================================

/// Set slash brightness (0-255).
pub fn set_slash_brightness(brightness: u8) -> Result<()> {
    run_asusctl(&["slash", "--brightness", &brightness.to_string()])?;
    Ok(())
}

/// Set slash mode.
pub fn set_slash_mode(mode: SlashMode) -> Result<()> {
    run_asusctl(&["slash", "--mode", &mode.to_string()])?;
    Ok(())
}

/// Set slash interval (0-5).
pub fn set_slash_interval(interval: u8) -> Result<()> {
    run_asusctl(&["slash", "--interval", &interval.to_string()])?;
    Ok(())
}

// ============================================================================
// Public API - State Getters (D-Bus preferred, config fallback)
// ============================================================================

/// Get slash enabled state (D-Bus preferred, config fallback).
pub fn get_slash_enabled() -> Result<bool> {
    get_slash_enabled_dbus().or_else(|_| Ok(parse_slash_config()?.enabled))
}

/// Get slash brightness (D-Bus preferred, config fallback).
pub fn get_slash_brightness() -> Result<u8> {
    get_slash_brightness_dbus().or_else(|_| Ok(parse_slash_config()?.brightness))
}

/// Get slash interval (D-Bus preferred, config fallback).
pub fn get_slash_interval() -> Result<u8> {
    get_slash_interval_dbus().or_else(|_| Ok(parse_slash_config()?.interval))
}

/// Get slash mode (from config file).
pub fn get_slash_mode() -> Result<SlashMode> {
    Ok(parse_slash_config()?.mode)
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
    Ok(())
}

pub fn set_slash_show_on_shutdown(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-shutdown",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

pub fn set_slash_show_on_sleep(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-sleep",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

pub fn set_slash_show_on_battery(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-battery",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

pub fn set_slash_show_battery_warning(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-battery-warning",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

// ============================================================================
// Private D-Bus Getters
// ============================================================================

fn get_slash_enabled_dbus() -> Result<bool> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "Enabled")?;
    parse_dbus_bool(&output)
}

fn get_slash_brightness_dbus() -> Result<u8> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "Brightness")?;
    parse_dbus_byte(&output)
}

fn get_slash_interval_dbus() -> Result<u8> {
    let path = get_slash_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Slash D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, SLASH_INTERFACE, "Interval")?;
    parse_dbus_byte(&output)
}

// ============================================================================
// Config File Parsing
// ============================================================================

/// Parse slash config from /etc/asusd/slash.ron.
fn parse_slash_config() -> Result<SlashState> {
    let content = fs::read_to_string(SLASH_CONFIG_PATH)
        .map_err(|e| AsusctlError::ParseError(format!("Failed to read slash config: {e}")))?;

    let mut state = SlashState::default();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("enabled:") {
            state.enabled = line.contains("true");
        } else if line.starts_with("brightness:") {
            if let Some(val) = extract_number(line) {
                state.brightness = val as u8;
            }
        } else if line.starts_with("display_interval:") {
            if let Some(val) = extract_number(line) {
                state.interval = val as u8;
            }
        } else if line.starts_with("display_mode:") {
            if let Some(mode_str) = extract_string_value(line) {
                state.mode = SlashMode::from_str(&mode_str).unwrap_or_default();
            }
        }
    }

    Ok(state)
}

/// Extract a number from a line like "brightness: 255,".
fn extract_number(line: &str) -> Option<u32> {
    line.split(':')
        .nth(1)?
        .trim()
        .trim_end_matches(',')
        .parse()
        .ok()
}

/// Extract a string value from a line like "display_mode: BitStream,".
fn extract_string_value(line: &str) -> Option<String> {
    Some(
        line.split(':')
            .nth(1)?
            .trim()
            .trim_end_matches(',')
            .to_string(),
    )
}
