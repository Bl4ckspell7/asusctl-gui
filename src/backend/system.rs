//! System information and feature detection.

use std::fs;
use std::str::FromStr;
use std::sync::OnceLock;

use super::dbus::{
    PLATFORM_INTERFACE, PLATFORM_PATH, get_aura_path, get_slash_path, has_armoury_interface,
    read_dbus_property_at, run_asusctl,
};
use super::error::{AsusctlError, Result};
use super::types::{AuraMode, KeyboardBrightness, SupportedFeatures, SystemInfo};

static DETECTED_FEATURES: OnceLock<SupportedFeatures> = OnceLock::new();

// ============================================================================
// Public API
// ============================================================================

/// Get system information (version, product family, board name).
pub fn get_system_info() -> Result<SystemInfo> {
    let output = run_asusctl(&["info"])?;
    parse_system_info(&output)
}

/// Get the Linux kernel version from /proc/sys/kernel/osrelease.
pub fn get_kernel_version() -> String {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Get the Linux distribution name from os-release (PRETTY_NAME field).
///
/// Tries `/run/host/os-release` first (available inside Flatpak) to get
/// the real host distro, then falls back to `/etc/os-release`.
pub fn get_distro() -> String {
    let content = fs::read_to_string("/run/host/os-release")
        .or_else(|_| fs::read_to_string("/etc/os-release"))
        .unwrap_or_default();
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            return value.trim_matches('"').to_string();
        }
    }
    String::new()
}

/// Detect available features once and cache the result for the process lifetime.
///
/// Tries `asusctl info --show-supported` first for the richest data.
/// Falls back to D-Bus probing when asusctl is not installed or the service
/// is not running.
pub fn detect_features() -> &'static SupportedFeatures {
    DETECTED_FEATURES.get_or_init(|| {
        let mut features = SupportedFeatures::default();

        match run_asusctl(&["info", "--show-supported"]) {
            Ok(output) => {
                features.asusctl_installed = true;
                features.asusd_running = true;
                if let Ok(parsed) = parse_supported_features(&output) {
                    let asusctl_installed = true;
                    let asusd_running = true;
                    features = parsed;
                    features.asusctl_installed = asusctl_installed;
                    features.asusd_running = asusd_running;
                }
            }
            Err(AsusctlError::NotInstalled) => {
                features.asusctl_installed = false;
                probe_dbus_features(&mut features);
            }
            Err(AsusctlError::ServiceNotRunning) => {
                features.asusctl_installed = true;
                features.asusd_running = false;
                probe_dbus_features(&mut features);
            }
            Err(_) => {
                features.asusctl_installed = true;
                probe_dbus_features(&mut features);
            }
        }

        log::info!(
            "Feature detection: asusctl={}, asusd={}, aura={}, platform={}, slash={}, charge_control={}, modes={}",
            features.asusctl_installed,
            features.asusd_running,
            features.has_aura,
            features.has_platform,
            features.has_slash,
            features.has_charge_control,
            features.aura_modes.len(),
        );

        features
    })
}

/// Probe D-Bus to discover available features without the asusctl CLI.
fn probe_dbus_features(features: &mut SupportedFeatures) {
    if get_aura_path().is_some() {
        features.has_aura = true;
    }

    if get_slash_path().is_some() {
        features.has_slash = true;
    }

    if read_dbus_property_at(
        PLATFORM_PATH,
        PLATFORM_INTERFACE,
        "ChargeControlEndThreshold",
    )
    .is_ok()
    {
        features.has_platform = true;
        features.has_charge_control = true;
    }
    if read_dbus_property_at(PLATFORM_PATH, PLATFORM_INTERFACE, "ThrottlePolicy").is_ok() {
        features.has_platform = true;
        features.has_throttle_policy = true;
    }

    features.has_armoury = has_armoury_interface();

    if features.has_aura || features.has_slash || features.has_platform {
        features.asusd_running = true;
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

fn parse_system_info(output: &str) -> Result<SystemInfo> {
    let mut info = SystemInfo::default();

    for line in output.lines() {
        let line = line.trim();

        // Handle both "Software version:" and "asusctl version:" formats
        if let Some(version) = line
            .strip_prefix("Software version:")
            .or_else(|| line.strip_prefix("asusctl version:"))
        {
            info.asusctl_version = version.trim().to_string();
        } else if let Some(family) = line.strip_prefix("Product family:") {
            info.product_family = family.trim().to_string();
        } else if let Some(board) = line.strip_prefix("Board name:") {
            info.board_name = board.trim().to_string();
        }
    }

    Ok(info)
}

fn parse_supported_features(output: &str) -> Result<SupportedFeatures> {
    let mut features = SupportedFeatures::default();

    // Parse core functions
    features.has_aura = output.contains("xyz.ljones.Aura");
    features.has_platform = output.contains("xyz.ljones.Platform");
    features.has_fan_curves = output.contains("xyz.ljones.FanCurves");
    features.has_slash = output.contains("xyz.ljones.Slash");
    features.has_armoury = output.contains("xyz.ljones.AsusArmoury");

    // Parse platform properties
    features.has_charge_control = output.contains("ChargeControlEndThreshold");
    features.has_throttle_policy = output.contains("ThrottlePolicy");

    // Parse keyboard brightness levels
    let brightness_section = extract_section(output, "Supported Keyboard Brightness:");
    for level in ["Off", "Low", "Med", "High"] {
        if brightness_section.contains(level) {
            if let Ok(brightness) = KeyboardBrightness::from_str(level) {
                features.keyboard_brightness_levels.push(brightness);
            }
        }
    }

    // Parse aura modes — match whole words to avoid substring false positives
    // (e.g. "Rain" matching inside "RainbowCycle")
    let aura_section = extract_section(output, "Supported Aura Modes:");
    for line in aura_section.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if trimmed.is_empty() || trimmed == "[" || trimmed == "]" {
            continue;
        }
        if let Ok(aura_mode) = AuraMode::from_str(trimmed) {
            if !features.aura_modes.contains(&aura_mode) {
                features.aura_modes.push(aura_mode);
            }
        }
    }

    // Parse aura zones
    let zones_section = extract_section(output, "Supported Aura Zones:");
    for line in zones_section.lines() {
        let trimmed = line
            .trim()
            .trim_matches(|c| c == ',' || c == '[' || c == ']');
        if !trimmed.is_empty() {
            features.aura_zones.push(trimmed.to_string());
        }
    }

    Ok(features)
}

/// Helper to extract a section from the output (between a header and the next header or end).
fn extract_section(output: &str, header: &str) -> String {
    let mut in_section = false;
    let mut section = String::new();
    let mut bracket_depth = 0;

    for line in output.lines() {
        if line.contains(header) {
            in_section = true;
            continue;
        }

        if in_section {
            // Track bracket depth to know when section ends
            bracket_depth += line.matches('[').count() as i32;
            bracket_depth -= line.matches(']').count() as i32;

            section.push_str(line);
            section.push('\n');

            // Section ends when we close all brackets and hit a new section
            if bracket_depth <= 0 && line.contains(']') {
                break;
            }
        }
    }

    section
}

#[cfg(test)]
#[path = "tests/system_tests.rs"]
mod tests;
