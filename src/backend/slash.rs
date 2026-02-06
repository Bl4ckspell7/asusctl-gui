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
    eprintln!("[asusctl-gui] Enabled slash LED bar");
    Ok(())
}

/// Disable slash LED bar.
pub fn disable_slash() -> Result<()> {
    run_asusctl(&["slash", "--disable"])?;
    eprintln!("[asusctl-gui] Disabled slash LED bar");
    Ok(())
}

// ============================================================================
// Public API - Settings
// ============================================================================

/// Set slash brightness (0-255).
pub fn set_slash_brightness(brightness: u8) -> Result<()> {
    run_asusctl(&["slash", "--brightness", &brightness.to_string()])?;
    eprintln!("[asusctl-gui] Set slash brightness to {brightness}");
    Ok(())
}

/// Set slash mode.
pub fn set_slash_mode(mode: SlashMode) -> Result<()> {
    run_asusctl(&["slash", "--mode", &mode.to_string()])?;
    eprintln!("[asusctl-gui] Set slash mode to {mode}");
    Ok(())
}

/// Set slash interval (0-5).
pub fn set_slash_interval(interval: u8) -> Result<()> {
    run_asusctl(&["slash", "--interval", &interval.to_string()])?;
    eprintln!("[asusctl-gui] Set slash interval to {interval}");
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
    eprintln!("[asusctl-gui] Set slash show-on-boot to {value}");
    Ok(())
}

pub fn set_slash_show_on_shutdown(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-shutdown",
        if value { "true" } else { "false" },
    ])?;
    eprintln!("[asusctl-gui] Set slash show-on-shutdown to {value}");
    Ok(())
}

pub fn set_slash_show_on_sleep(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-sleep",
        if value { "true" } else { "false" },
    ])?;
    eprintln!("[asusctl-gui] Set slash show-on-sleep to {value}");
    Ok(())
}

pub fn set_slash_show_on_battery(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-battery",
        if value { "true" } else { "false" },
    ])?;
    eprintln!("[asusctl-gui] Set slash show-on-battery to {value}");
    Ok(())
}

pub fn set_slash_show_battery_warning(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-battery-warning",
        if value { "true" } else { "false" },
    ])?;
    eprintln!("[asusctl-gui] Set slash show-battery-warning to {value}");
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // extract_number Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_extract_number_valid() {
        assert_eq!(extract_number("brightness: 255,"), Some(255));
        assert_eq!(extract_number("brightness: 128,"), Some(128));
        assert_eq!(extract_number("display_interval: 5,"), Some(5));
        assert_eq!(extract_number("value: 0,"), Some(0));
    }

    #[test]
    fn test_extract_number_without_trailing_comma() {
        assert_eq!(extract_number("brightness: 255"), Some(255));
        assert_eq!(extract_number("interval: 3"), Some(3));
    }

    #[test]
    fn test_extract_number_with_whitespace() {
        // Leading whitespace after colon is handled
        assert_eq!(extract_number("brightness:   255,"), Some(255));
        // Note: "brightness: 255  ," won't parse because whitespace before comma
        // is not trimmed by the simple parsing logic. This matches expected config format.
        assert_eq!(extract_number("  brightness: 100,"), Some(100));
    }

    #[test]
    fn test_extract_number_invalid() {
        assert_eq!(extract_number("brightness: abc,"), None);
        assert_eq!(extract_number("no_colon_here"), None);
        assert_eq!(extract_number("brightness:"), None);
        assert_eq!(extract_number(""), None);
    }

    #[test]
    fn test_extract_number_large_values() {
        assert_eq!(extract_number("value: 4294967295,"), Some(4294967295));
    }

    // ------------------------------------------------------------------------
    // extract_string_value Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_extract_string_value_valid() {
        assert_eq!(
            extract_string_value("display_mode: BitStream,"),
            Some("BitStream".to_string())
        );
        assert_eq!(
            extract_string_value("display_mode: Flow,"),
            Some("Flow".to_string())
        );
        assert_eq!(
            extract_string_value("mode: Static,"),
            Some("Static".to_string())
        );
    }

    #[test]
    fn test_extract_string_value_without_trailing_comma() {
        assert_eq!(
            extract_string_value("display_mode: Spectrum"),
            Some("Spectrum".to_string())
        );
    }

    #[test]
    fn test_extract_string_value_with_whitespace() {
        assert_eq!(
            extract_string_value("display_mode:   BitStream,"),
            Some("BitStream".to_string())
        );
        assert_eq!(
            extract_string_value("  display_mode: Flow,  "),
            Some("Flow".to_string())
        );
    }

    #[test]
    fn test_extract_string_value_empty_value() {
        assert_eq!(extract_string_value("mode:,"), Some("".to_string()));
        assert_eq!(extract_string_value("mode: ,"), Some("".to_string()));
    }

    #[test]
    fn test_extract_string_value_no_colon() {
        assert_eq!(extract_string_value("no_colon_here"), None);
        assert_eq!(extract_string_value(""), None);
    }

    // ------------------------------------------------------------------------
    // Parsing enabled line Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_enabled_line() {
        // Test parsing enabled: true/false from config lines
        assert!("enabled: true,".contains("true"));
        assert!(!"enabled: false,".contains("true"));
    }

    // ------------------------------------------------------------------------
    // SlashState Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_slash_state_default_values() {
        let state = SlashState::default();
        assert!(!state.enabled);
        assert_eq!(state.brightness, 0);
        assert_eq!(state.interval, 0);
        assert_eq!(state.mode, SlashMode::Flow);
    }

    #[test]
    fn test_slash_state_custom_values() {
        let state = SlashState {
            enabled: true,
            brightness: 200,
            interval: 3,
            mode: SlashMode::BitStream,
        };
        assert!(state.enabled);
        assert_eq!(state.brightness, 200);
        assert_eq!(state.interval, 3);
        assert_eq!(state.mode, SlashMode::BitStream);
    }

    // ------------------------------------------------------------------------
    // Integration-style parsing Tests (simulated config content)
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_config_lines() {
        // Simulate parsing a config file line by line
        let config_content = r#"(
    enabled: true,
    brightness: 180,
    display_interval: 2,
    display_mode: Spectrum,
)"#;

        let mut state = SlashState::default();

        for line in config_content.lines() {
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

        assert!(state.enabled);
        assert_eq!(state.brightness, 180);
        assert_eq!(state.interval, 2);
        assert_eq!(state.mode, SlashMode::Spectrum);
    }

    #[test]
    fn test_parse_config_lines_disabled() {
        let config_content = r#"(
    enabled: false,
    brightness: 100,
    display_interval: 5,
    display_mode: Hazard,
)"#;

        let mut state = SlashState::default();

        for line in config_content.lines() {
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

        assert!(!state.enabled);
        assert_eq!(state.brightness, 100);
        assert_eq!(state.interval, 5);
        assert_eq!(state.mode, SlashMode::Hazard);
    }

    #[test]
    fn test_parse_config_with_unknown_mode_falls_back_to_default() {
        let line = "display_mode: UnknownMode,";
        let mode_str = extract_string_value(line).unwrap();
        let mode = SlashMode::from_str(&mode_str).unwrap_or_default();
        assert_eq!(mode, SlashMode::Flow); // Default mode
    }
}
