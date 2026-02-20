//! Power profile and charge control management.

use std::str::FromStr;

use super::dbus::{
    PLATFORM_INTERFACE, PLATFORM_PATH, host_command, parse_dbus_byte, read_dbus_property_at,
    run_asusctl,
};
use super::error::{AsusctlError, Result};
use super::types::{PowerProfile, ProfileState};

// ============================================================================
// Public API - Power Profiles
// ============================================================================

/// Get current profile state (active, on AC, on battery) via CLI.
pub fn get_profile_state() -> Result<ProfileState> {
    let output = run_asusctl(&["profile", "get"])?;
    parse_profile_state(&output)
}

/// Set the active power profile using powerprofilesctl (preferred) or asusctl (fallback).
///
/// Uses power-profiles-daemon when available to maintain GNOME integration.
/// Falls back to asusctl if powerprofilesctl is not installed.
pub fn set_profile(profile: PowerProfile) -> Result<()> {
    // Try powerprofilesctl first for GNOME integration
    if set_profile_ppdctl(profile).is_ok() {
        log::info!("Set power profile to {profile}, using powerprofilesctl");
        return Ok(());
    }

    // Fall back to asusctl
    run_asusctl(&["profile", "set", &profile.to_string()])?;
    log::info!("Set power profile to {profile}, using asusctl");
    Ok(())
}

/// Set the power profile to use when on AC power.
pub fn set_profile_ac(profile: PowerProfile) -> Result<()> {
    run_asusctl(&["profile", "set", &profile.to_string(), "--ac"])?;
    log::info!("Set AC power profile to {profile}");
    Ok(())
}

/// Set the power profile to use when on battery power.
pub fn set_profile_battery(profile: PowerProfile) -> Result<()> {
    run_asusctl(&["profile", "set", &profile.to_string(), "--battery"])?;
    log::info!("Set battery power profile to {profile}");
    Ok(())
}

// ============================================================================
// Public API - Charge Control
// ============================================================================

/// Get charge control threshold via D-Bus.
pub fn get_charge_limit_dbus() -> Result<u8> {
    let output = read_dbus_property_at(
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "ChargeControlEndThreshold",
    )?;
    parse_dbus_byte(&output)
}

/// Set charge limit (20-100).
pub fn set_charge_limit(limit: u8) -> Result<()> {
    run_asusctl(&["battery", "limit", &limit.to_string()])?;
    log::info!("Set charge limit to {limit}%");
    Ok(())
}

// ============================================================================
// Private Helpers
// ============================================================================

/// Set profile using powerprofilesctl.
fn set_profile_ppdctl(profile: PowerProfile) -> Result<()> {
    let profile_name = match profile {
        PowerProfile::Quiet => "power-saver",
        PowerProfile::Balanced => "balanced",
        PowerProfile::Performance => "performance",
    };

    let output = host_command("powerprofilesctl")
        .args(["set", profile_name])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AsusctlError::NotInstalled
            } else {
                AsusctlError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AsusctlError::CommandFailed(stderr.to_string()));
    }

    Ok(())
}

fn parse_profile_state(output: &str) -> Result<ProfileState> {
    let mut state = ProfileState::default();

    for line in output.lines() {
        let line = line.trim();

        // Handle "Active profile: Quiet" or "Active profile is Quiet" formats
        if line.starts_with("Active profile") {
            let profile_str = line
                .strip_prefix("Active profile:")
                .or_else(|| line.strip_prefix("Active profile is "));
            if let Some(profile) = profile_str {
                state.active = PowerProfile::from_str(profile.trim())?;
            }
        }
        // Handle "AC profile Quiet" or "Profile on AC is Quiet" formats
        else if line.starts_with("AC profile") {
            if let Some(profile) = line.strip_prefix("AC profile") {
                state.on_ac = PowerProfile::from_str(profile.trim())?;
            }
        } else if line.starts_with("Profile on AC") {
            if let Some(profile) = line.strip_prefix("Profile on AC is ") {
                state.on_ac = PowerProfile::from_str(profile.trim())?;
            }
        }
        // Handle "Battery profile Quiet" or "Profile on Battery is Quiet" formats
        else if line.starts_with("Battery profile") {
            if let Some(profile) = line.strip_prefix("Battery profile") {
                state.on_battery = PowerProfile::from_str(profile.trim())?;
            }
        } else if line.starts_with("Profile on Battery") {
            if let Some(profile) = line.strip_prefix("Profile on Battery is ") {
                state.on_battery = PowerProfile::from_str(profile.trim())?;
            }
        }
    }

    Ok(state)
}

#[cfg(test)]
#[path = "tests/power_tests.rs"]
mod tests;
