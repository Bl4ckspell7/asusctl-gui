//! Aura lighting and keyboard brightness control.

use std::str::FromStr;

use super::dbus::{
    AURA_INTERFACE, get_aura_path, parse_dbus_uint, read_dbus_properties_at, read_dbus_property_at,
    run_asusctl,
};
use super::error::{AsusctlError, Result};
use super::types::{
    AuraDirection, AuraMode, AuraModeData, AuraPowerState, AuraPowerZone, AuraSpeed,
    KeyboardBrightness,
};

/// `DeviceType` of 2021+ laptop keyboards, the only ones `asusctl aura power` handles.
const LAPTOP_KEYBOARD_2021: u32 = 0;

/// D-Bus signature of the LedPower property.
const LED_POWER_SIGNATURE: &str = "(a(ubbbb))";

// ============================================================================
// Public API - Keyboard Brightness
// ============================================================================

/// Get current keyboard brightness via D-Bus.
pub fn get_keyboard_brightness_dbus() -> Result<KeyboardBrightness> {
    let path = aura_path()?;
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
    log::info!("Set keyboard brightness to {level}");
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
    let path = aura_path()?;
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
        let c = colour
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires a colour")))?;
        args.extend_from_slice(&["--colour", c]);
    }
    if mode.needs_colour2() {
        let c2 = colour2
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires colour2")))?;
        args.extend_from_slice(&["--colour2", c2]);
    }
    if mode.needs_direction() {
        let d = direction_str
            .as_deref()
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires direction")))?;
        args.extend_from_slice(&["--direction", d]);
    }
    if mode.needs_speed() {
        let s = speed_str
            .as_deref()
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires speed")))?;
        args.extend_from_slice(&["--speed", s]);
    }
    if let Some(z) = zone {
        args.extend_from_slice(&["--zone", z]);
    }

    run_asusctl(&args)?;
    log::info!("Set aura mode to {mode}");
    Ok(())
}

/// Fetch the help text for a given aura mode from the CLI.
pub fn get_aura_mode_help(mode: AuraMode) -> Option<String> {
    run_asusctl(&["aura", "effect", mode.cli_name(), "--help"])
        .ok()
        .map(|s| s.trim().to_string())
}

// ============================================================================
// Public API - Power States
// ============================================================================

/// Get when each lighting zone is on (boot, awake, sleep, shutdown) via D-Bus.
pub fn get_aura_power_states() -> Result<Vec<AuraPowerState>> {
    let path = aura_path()?;
    let output = read_dbus_property_at(path, AURA_INTERFACE, "LedPower")?;
    parse_led_power(&output)
}

/// Set when a zone's lighting is on.
///
/// The CLI writes all four states of the zone at once: an omitted flag means off.
pub fn set_aura_power_state(state: AuraPowerState) -> Result<()> {
    run_asusctl(&aura_power_args(&state))?;
    log::info!(
        "Set {} lighting power states to {state:?}",
        state.zone.label()
    );
    Ok(())
}

/// Zones whose power states can be set, or none if this keyboard isn't supported.
///
/// `asusctl aura power` only handles 2021+ keyboards; for other device types it
/// exits successfully without changing anything.
pub(super) fn detect_aura_power_zones() -> Vec<AuraPowerZone> {
    let Some(path) = get_aura_path() else {
        return Vec::new();
    };

    let values = match read_dbus_properties_at(path, AURA_INTERFACE, &["DeviceType", "LedPower"]) {
        Ok(values) => values,
        Err(e) => {
            log::warn!("Failed to read Aura device type and power states: {e}");
            return Vec::new();
        }
    };

    if parse_dbus_uint(&values[0]).ok() != Some(LAPTOP_KEYBOARD_2021) {
        return Vec::new();
    }

    match parse_led_power(&values[1]) {
        Ok(states) => states.into_iter().map(|state| state.zone).collect(),
        Err(e) => {
            log::warn!("Failed to parse Aura power states: {e}");
            Vec::new()
        }
    }
}

// ============================================================================
// Private Helpers
// ============================================================================

/// The Aura D-Bus path of the keyboard.
fn aura_path() -> Result<&'static String> {
    get_aura_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Aura D-Bus path not found".to_string()))
}

/// Parse the LedPower property.
///
/// busctl prints `(a(ubbbb)) 1 1 true true true true`: the zone count, then
/// `zone boot awake sleep shutdown` per zone. Zones this app doesn't know are skipped.
fn parse_led_power(output: &str) -> Result<Vec<AuraPowerState>> {
    let parse_error = || AsusctlError::ParseError(format!("Invalid LedPower format: {output}"));

    let mut tokens = output.split_whitespace();
    if tokens.next() != Some(LED_POWER_SIGNATURE) {
        return Err(parse_error());
    }

    let count: usize = tokens
        .next()
        .and_then(|token| token.parse().ok())
        .ok_or_else(parse_error)?;
    let values: Vec<&str> = tokens.collect();
    if Some(values.len()) != count.checked_mul(5) {
        return Err(parse_error());
    }

    let parse_bool = |token: &str| match token {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(parse_error()),
    };

    let mut states = Vec::new();
    for chunk in values.chunks(5) {
        let zone_id: u32 = chunk[0].parse().map_err(|_| parse_error())?;
        let boot = parse_bool(chunk[1])?;
        let awake = parse_bool(chunk[2])?;
        let sleep = parse_bool(chunk[3])?;
        let shutdown = parse_bool(chunk[4])?;

        if let Some(zone) = AuraPowerZone::from_dbus_value(zone_id) {
            states.push(AuraPowerState {
                zone,
                boot,
                awake,
                sleep,
                shutdown,
            });
        }
    }

    Ok(states)
}

/// Build `asusctl aura power <zone> [--boot] [--awake] [--sleep] [--shutdown]`.
fn aura_power_args(state: &AuraPowerState) -> Vec<&'static str> {
    let mut args = vec!["aura", "power", state.zone.cli_name()];
    for (enabled, flag) in [
        (state.boot, "--boot"),
        (state.awake, "--awake"),
        (state.sleep, "--sleep"),
        (state.shutdown, "--shutdown"),
    ] {
        if enabled {
            args.push(flag);
        }
    }
    args
}

#[cfg(test)]
#[path = "tests/aura_tests.rs"]
mod tests;
