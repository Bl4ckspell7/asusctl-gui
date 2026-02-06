//! System information and feature detection.

use std::str::FromStr;
use std::sync::OnceLock;

use super::dbus::{
    PLATFORM_INTERFACE, PLATFORM_PATH, get_aura_path, get_slash_path, read_dbus_property_at,
    run_asusctl,
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

        eprintln!(
            "[asusctl-gui] Feature detection: asusctl={}, asusd={}, aura={}, platform={}, slash={}, charge_control={}, modes={}",
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // parse_system_info Tests
    // ------------------------------------------------------------------------

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

    #[test]
    fn test_parse_system_info_software_version_format() {
        let output = r#"Software version: 6.1.0
Product family: ROG Flow X13
Board name: GV302XA"#;

        let info = parse_system_info(output).unwrap();
        assert_eq!(info.asusctl_version, "6.1.0");
        assert_eq!(info.product_family, "ROG Flow X13");
        assert_eq!(info.board_name, "GV302XA");
    }

    #[test]
    fn test_parse_system_info_empty_output() {
        let output = "";
        let info = parse_system_info(output).unwrap();
        assert!(info.asusctl_version.is_empty());
        assert!(info.product_family.is_empty());
        assert!(info.board_name.is_empty());
    }

    #[test]
    fn test_parse_system_info_partial_output() {
        let output = "asusctl version: 6.2.0";
        let info = parse_system_info(output).unwrap();
        assert_eq!(info.asusctl_version, "6.2.0");
        assert!(info.product_family.is_empty());
        assert!(info.board_name.is_empty());
    }

    // ------------------------------------------------------------------------
    // extract_section Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_extract_section_simple() {
        let output = r#"Some header info
Supported Keyboard Brightness:
[
    Off,
    Low,
    Med,
    High,
]
Other section:"#;

        let section = extract_section(output, "Supported Keyboard Brightness:");
        assert!(section.contains("Off"));
        assert!(section.contains("Low"));
        assert!(section.contains("Med"));
        assert!(section.contains("High"));
    }

    #[test]
    fn test_extract_section_not_found() {
        let output = "Some content without the header";
        let section = extract_section(output, "Nonexistent Header:");
        assert!(section.is_empty());
    }

    #[test]
    fn test_extract_section_with_aura_modes() {
        let output = r#"Supported Aura Modes:
[
    Static,
    Breathe,
    RainbowCycle,
    Stars,
]
Next section:"#;

        let section = extract_section(output, "Supported Aura Modes:");
        assert!(section.contains("Static"));
        assert!(section.contains("Breathe"));
        assert!(section.contains("RainbowCycle"));
        assert!(section.contains("Stars"));
    }

    // ------------------------------------------------------------------------
    // parse_supported_features Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_parse_supported_features_full() {
        let output = r#"Core functions:
  xyz.ljones.Aura
  xyz.ljones.Platform
  xyz.ljones.FanCurves
  xyz.ljones.Slash

Platform properties:
  ChargeControlEndThreshold
  ThrottlePolicy

Supported Keyboard Brightness:
[
    Off,
    Low,
    Med,
    High,
]

Supported Aura Modes:
[
    Static,
    Breathe,
    RainbowCycle,
    RainbowWave,
    Stars,
    Rain,
]

Supported Aura Zones:
[
    Key1,
    Key2,
]
"#;

        let features = parse_supported_features(output).unwrap();

        // Core functions
        assert!(features.has_aura);
        assert!(features.has_platform);
        assert!(features.has_fan_curves);
        assert!(features.has_slash);

        // Platform properties
        assert!(features.has_charge_control);
        assert!(features.has_throttle_policy);

        // Keyboard brightness
        assert_eq!(features.keyboard_brightness_levels.len(), 4);
        assert!(
            features
                .keyboard_brightness_levels
                .contains(&KeyboardBrightness::Off)
        );
        assert!(
            features
                .keyboard_brightness_levels
                .contains(&KeyboardBrightness::High)
        );

        // Aura modes
        assert!(features.aura_modes.len() >= 6);
        assert!(features.aura_modes.contains(&AuraMode::Static));
        assert!(features.aura_modes.contains(&AuraMode::Breathe));
        assert!(features.aura_modes.contains(&AuraMode::RainbowCycle));
        assert!(features.aura_modes.contains(&AuraMode::RainbowWave));
        assert!(features.aura_modes.contains(&AuraMode::Stars));
        assert!(features.aura_modes.contains(&AuraMode::Rain));

        // Aura zones
        assert_eq!(features.aura_zones.len(), 2);
    }

    #[test]
    fn test_parse_supported_features_minimal() {
        let output = "xyz.ljones.Aura";
        let features = parse_supported_features(output).unwrap();
        assert!(features.has_aura);
        assert!(!features.has_platform);
        assert!(!features.has_slash);
    }

    #[test]
    fn test_parse_supported_features_no_aura() {
        let output = r#"Core functions:
  xyz.ljones.Platform
  xyz.ljones.Slash
"#;
        let features = parse_supported_features(output).unwrap();
        assert!(!features.has_aura);
        assert!(features.has_platform);
        assert!(features.has_slash);
    }

    #[test]
    fn test_parse_supported_features_empty() {
        let output = "";
        let features = parse_supported_features(output).unwrap();
        assert!(!features.has_aura);
        assert!(!features.has_platform);
        assert!(!features.has_slash);
        assert!(features.keyboard_brightness_levels.is_empty());
        assert!(features.aura_modes.is_empty());
    }

    #[test]
    fn test_parse_supported_features_all_aura_modes() {
        let output = r#"Supported Aura Modes:
[
    Static,
    Breathe,
    RainbowCycle,
    RainbowWave,
    Stars,
    Rain,
    Highlight,
    Laser,
    Ripple,
    Pulse,
    Comet,
    Flash,
]
"#;

        let features = parse_supported_features(output).unwrap();
        assert_eq!(features.aura_modes.len(), 12);
    }

    #[test]
    fn test_parse_supported_features_no_duplicate_modes() {
        // Test that modes are not duplicated if they appear multiple times
        let output = r#"Supported Aura Modes:
[
    Static,
    Static,
    Breathe,
]
"#;

        let features = parse_supported_features(output).unwrap();
        // Should only have 2 unique modes
        let static_count = features
            .aura_modes
            .iter()
            .filter(|m| **m == AuraMode::Static)
            .count();
        assert_eq!(static_count, 1);
    }
}
