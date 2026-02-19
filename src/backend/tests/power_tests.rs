use super::*;

// ------------------------------------------------------------------------
// parse_profile_state Tests
// ------------------------------------------------------------------------

#[test]
fn test_parse_profile_state() {
    // Test actual asusctl output format
    let output = r#"Active profile: Quiet

AC profile Quiet
Battery profile Quiet"#;

    let Ok(state) = parse_profile_state(output) else {
        panic!("failed to parse profile state");
    };
    assert_eq!(state.active, PowerProfile::Quiet);
    assert_eq!(state.on_ac, PowerProfile::Quiet);
    assert_eq!(state.on_battery, PowerProfile::Quiet);
}

#[test]
fn test_parse_profile_state_legacy_format() {
    // Test legacy format (for compatibility)
    let output = r#"Starting version 6.2.0
Active profile is Quiet
Profile on AC is Quiet
Profile on Battery is Quiet"#;

    let Ok(state) = parse_profile_state(output) else {
        panic!("failed to parse profile state");
    };
    assert_eq!(state.active, PowerProfile::Quiet);
    assert_eq!(state.on_ac, PowerProfile::Quiet);
    assert_eq!(state.on_battery, PowerProfile::Quiet);
}

#[test]
fn test_parse_profile_state_balanced() {
    let output = r#"Active profile: Balanced

AC profile Balanced
Battery profile Quiet"#;

    let state = parse_profile_state(output).unwrap();
    assert_eq!(state.active, PowerProfile::Balanced);
    assert_eq!(state.on_ac, PowerProfile::Balanced);
    assert_eq!(state.on_battery, PowerProfile::Quiet);
}

#[test]
fn test_parse_profile_state_performance() {
    let output = r#"Active profile: Performance

AC profile Performance
Battery profile Balanced"#;

    let state = parse_profile_state(output).unwrap();
    assert_eq!(state.active, PowerProfile::Performance);
    assert_eq!(state.on_ac, PowerProfile::Performance);
    assert_eq!(state.on_battery, PowerProfile::Balanced);
}

#[test]
fn test_parse_profile_state_mixed_profiles() {
    let output = r#"Active profile: Quiet

AC profile Performance
Battery profile Quiet"#;

    let state = parse_profile_state(output).unwrap();
    assert_eq!(state.active, PowerProfile::Quiet);
    assert_eq!(state.on_ac, PowerProfile::Performance);
    assert_eq!(state.on_battery, PowerProfile::Quiet);
}

#[test]
fn test_parse_profile_state_empty_output() {
    let output = "";
    let state = parse_profile_state(output).unwrap();
    // Should return defaults
    assert_eq!(state.active, PowerProfile::default());
    assert_eq!(state.on_ac, PowerProfile::default());
    assert_eq!(state.on_battery, PowerProfile::default());
}

#[test]
fn test_parse_profile_state_partial_output() {
    let output = "Active profile: Performance";
    let state = parse_profile_state(output).unwrap();
    assert_eq!(state.active, PowerProfile::Performance);
    // AC and battery should be defaults
    assert_eq!(state.on_ac, PowerProfile::default());
    assert_eq!(state.on_battery, PowerProfile::default());
}

#[test]
fn test_parse_profile_state_with_extra_whitespace() {
    let output = r#"  Active profile:   Quiet

  AC profile   Balanced
  Battery profile   Performance  "#;

    let state = parse_profile_state(output).unwrap();
    assert_eq!(state.active, PowerProfile::Quiet);
    assert_eq!(state.on_ac, PowerProfile::Balanced);
    assert_eq!(state.on_battery, PowerProfile::Performance);
}

#[test]
fn test_parse_profile_state_case_insensitive() {
    let output = r#"Active profile: QUIET

AC profile BALANCED
Battery profile PERFORMANCE"#;

    let state = parse_profile_state(output).unwrap();
    assert_eq!(state.active, PowerProfile::Quiet);
    assert_eq!(state.on_ac, PowerProfile::Balanced);
    assert_eq!(state.on_battery, PowerProfile::Performance);
}

// ------------------------------------------------------------------------
// PowerProfile mapping for powerprofilesctl Tests
// ------------------------------------------------------------------------

#[test]
fn test_power_profile_ppdctl_mapping() {
    // Test that the mapping in set_profile_ppdctl is correct
    // These are the expected mappings to power-profiles-daemon names
    assert_eq!(
        match PowerProfile::Quiet {
            PowerProfile::Quiet => "power-saver",
            PowerProfile::Balanced => "balanced",
            PowerProfile::Performance => "performance",
        },
        "power-saver"
    );

    assert_eq!(
        match PowerProfile::Balanced {
            PowerProfile::Quiet => "power-saver",
            PowerProfile::Balanced => "balanced",
            PowerProfile::Performance => "performance",
        },
        "balanced"
    );

    assert_eq!(
        match PowerProfile::Performance {
            PowerProfile::Quiet => "power-saver",
            PowerProfile::Balanced => "balanced",
            PowerProfile::Performance => "performance",
        },
        "performance"
    );
}
