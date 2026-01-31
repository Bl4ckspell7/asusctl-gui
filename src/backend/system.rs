//! System information and feature detection.

use std::str::FromStr;

use super::dbus::run_asusctl;
use super::error::Result;
use super::types::{AuraMode, KeyboardBrightness, SupportedFeatures, SystemInfo};

// ============================================================================
// Public API
// ============================================================================

/// Get system information (version, product family, board name).
pub fn get_system_info() -> Result<SystemInfo> {
    let output = run_asusctl(&["info"])?;
    parse_system_info(&output)
}

/// Get supported features for this laptop.
pub fn get_supported_features() -> Result<SupportedFeatures> {
    let output = run_asusctl(&["info", "--show-supported"])?;
    parse_supported_features(&output)
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

    // Parse aura modes
    let aura_section = extract_section(output, "Supported Aura Modes:");
    for mode_name in [
        "Static",
        "Breathe",
        "RainbowCycle",
        "RainbowWave",
        "Stars",
        "Rain",
        "Highlight",
        "Laser",
        "Ripple",
        "Pulse",
        "Comet",
        "Flash",
    ] {
        if aura_section.contains(mode_name) {
            if let Ok(aura_mode) = AuraMode::from_str(mode_name) {
                if !features.aura_modes.contains(&aura_mode) {
                    features.aura_modes.push(aura_mode);
                }
            }
        }
    }

    // Parse aura zones
    let zones_section = extract_section(output, "Supported Aura Zones:");
    for line in zones_section.lines() {
        let trimmed = line.trim().trim_matches(|c| c == ',' || c == '[' || c == ']');
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_system_info() {
        let output = r#"Starting version 6.2.0
asusctl v6.2.0
asusctl version: 6.2.0
 Product family: ROG Zephyrus G14
     Board name: GA403UV"#;

        let Ok(info) = parse_system_info(output) else {
            panic!("failed to parse system info");
        };
        assert_eq!(info.asusctl_version, "6.2.0");
        assert_eq!(info.product_family, "ROG Zephyrus G14");
        assert_eq!(info.board_name, "GA403UV");
    }
}
