//! Aura lighting and keyboard brightness control.

use std::str::FromStr;

use super::dbus::{
    get_aura_path, parse_dbus_uint, read_dbus_property_at, run_asusctl, AURA_INTERFACE,
};
use super::error::{AsusctlError, Result};
use super::types::{AuraDirection, AuraMode, AuraModeData, AuraSpeed, KeyboardBrightness};

// ============================================================================
// Public API - Keyboard Brightness
// ============================================================================

/// Get current keyboard brightness via D-Bus.
pub fn get_keyboard_brightness_dbus() -> Result<KeyboardBrightness> {
    let path = get_aura_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Aura D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, AURA_INTERFACE, "Brightness")?;
    let value = parse_dbus_uint(&output)?;

    match value {
        0 => Ok(KeyboardBrightness::Off),
        1 => Ok(KeyboardBrightness::Low),
        2 => Ok(KeyboardBrightness::Med),
        3 => Ok(KeyboardBrightness::High),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown brightness value: {value}"
        ))),
    }
}

/// Set keyboard brightness level.
pub fn set_keyboard_brightness(level: KeyboardBrightness) -> Result<()> {
    run_asusctl(&["leds", "set", &level.to_string()])?;
    eprintln!("[asusctl-gui] Set keyboard brightness to {level}");
    Ok(())
}

// ============================================================================
// Public API - Aura Mode
// ============================================================================

/// Get full aura mode data (mode, colors, speed, direction) via D-Bus.
///
/// Parses the LedModeData property which has signature (uu(yyy)(yyy)ss):
/// (mode, zone, (r1,g1,b1), (r2,g2,b2), speed, direction)
pub fn get_aura_mode_data_dbus() -> Result<AuraModeData> {
    let path = get_aura_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Aura D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, AURA_INTERFACE, "LedModeData")?;

    // Parse output like: (uu(yyy)(yyy)ss) 1 0 255 0 255 255 0 0 "Med" "Right"
    // After the signature, values are space-separated
    let parts: Vec<&str> = output.split_whitespace().collect();

    // Expected: signature + mode + zone + r1 + g1 + b1 + r2 + g2 + b2 + speed + direction
    if parts.len() < 11 {
        return Err(AsusctlError::ParseError(format!(
            "Invalid LedModeData format: {output}"
        )));
    }

    let mode_val: u32 = parts[1]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid mode value".to_string()))?;
    let zone: u32 = parts[2]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid zone value".to_string()))?;
    let r1: u8 = parts[3]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color1 R".to_string()))?;
    let g1: u8 = parts[4]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color1 G".to_string()))?;
    let b1: u8 = parts[5]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color1 B".to_string()))?;
    let r2: u8 = parts[6]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color2 R".to_string()))?;
    let g2: u8 = parts[7]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color2 G".to_string()))?;
    let b2: u8 = parts[8]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color2 B".to_string()))?;

    // Speed and direction are quoted strings like "Med" "Right"
    let speed_str = parts[9].trim_matches('"');
    let direction_str = parts[10].trim_matches('"');

    let mode = AuraMode::from_dbus_value(mode_val).unwrap_or_default();
    let speed = AuraSpeed::from_str(speed_str).unwrap_or_default();
    let direction = AuraDirection::from_str(direction_str).unwrap_or_default();

    Ok(AuraModeData {
        mode,
        zone,
        color1: (r1, g1, b1),
        color2: (r2, g2, b2),
        speed,
        direction,
    })
}

/// Set aura lighting mode with the given parameters.
pub fn set_aura_mode(
    mode: AuraMode,
    colour: Option<&str>,
    colour2: Option<&str>,
    speed: Option<AuraSpeed>,
    direction: Option<AuraDirection>,
    zone: Option<&str>,
) -> Result<()> {
    let mode_name = mode.cli_name();
    let speed_str = speed.map(|s| s.to_string());
    let direction_str = direction.map(|d| d.to_string());

    let mut args: Vec<&str> = vec!["aura", "effect", mode_name];

    if mode.needs_colour() {
        let c = colour.ok_or_else(|| {
            AsusctlError::CommandFailed(format!("{mode} requires a colour"))
        })?;
        args.extend_from_slice(&["--colour", c]);
    }
    if mode.needs_colour2() {
        let c2 = colour2.ok_or_else(|| {
            AsusctlError::CommandFailed(format!("{mode} requires colour2"))
        })?;
        args.extend_from_slice(&["--colour2", c2]);
    }
    if mode.needs_direction() {
        let d = direction_str.as_deref().ok_or_else(|| {
            AsusctlError::CommandFailed(format!("{mode} requires direction"))
        })?;
        args.extend_from_slice(&["--direction", d]);
    }
    if mode.needs_speed() {
        let s = speed_str.as_deref().ok_or_else(|| {
            AsusctlError::CommandFailed(format!("{mode} requires speed"))
        })?;
        args.extend_from_slice(&["--speed", s]);
    }
    if let Some(z) = zone {
        args.extend_from_slice(&["--zone", z]);
    }

    run_asusctl(&args)?;
    eprintln!("[asusctl-gui] Set aura mode to {mode}");
    Ok(())
}

/// Fetch the help text for a given aura mode from the CLI.
pub fn get_aura_mode_help(mode: AuraMode) -> Option<String> {
    run_asusctl(&["aura", "effect", mode.cli_name(), "--help"])
        .ok()
        .map(|s| s.trim().to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_from_str() {
        assert!(matches!(
            KeyboardBrightness::from_str("High"),
            Ok(KeyboardBrightness::High)
        ));
        assert!(matches!(
            KeyboardBrightness::from_str("off"),
            Ok(KeyboardBrightness::Off)
        ));
    }
}
