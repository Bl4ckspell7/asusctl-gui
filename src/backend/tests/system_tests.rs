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
