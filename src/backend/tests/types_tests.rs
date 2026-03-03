use super::*;

// ------------------------------------------------------------------------
// KeyboardBrightness Tests
// ------------------------------------------------------------------------

#[test]
fn test_keyboard_brightness_from_str_valid() {
    assert_eq!(
        KeyboardBrightness::from_str("off").unwrap(),
        KeyboardBrightness::Off
    );
    assert_eq!(
        KeyboardBrightness::from_str("OFF").unwrap(),
        KeyboardBrightness::Off
    );
    assert_eq!(
        KeyboardBrightness::from_str("low").unwrap(),
        KeyboardBrightness::Low
    );
    assert_eq!(
        KeyboardBrightness::from_str("Low").unwrap(),
        KeyboardBrightness::Low
    );
    assert_eq!(
        KeyboardBrightness::from_str("med").unwrap(),
        KeyboardBrightness::Med
    );
    assert_eq!(
        KeyboardBrightness::from_str("MED").unwrap(),
        KeyboardBrightness::Med
    );
    assert_eq!(
        KeyboardBrightness::from_str("high").unwrap(),
        KeyboardBrightness::High
    );
    assert_eq!(
        KeyboardBrightness::from_str("HIGH").unwrap(),
        KeyboardBrightness::High
    );
}

#[test]
fn test_keyboard_brightness_from_str_invalid() {
    assert!(KeyboardBrightness::from_str("invalid").is_err());
    assert!(KeyboardBrightness::from_str("").is_err());
    assert!(KeyboardBrightness::from_str("medium").is_err());
}

#[test]
fn test_keyboard_brightness_display() {
    assert_eq!(KeyboardBrightness::Off.to_string(), "off");
    assert_eq!(KeyboardBrightness::Low.to_string(), "low");
    assert_eq!(KeyboardBrightness::Med.to_string(), "med");
    assert_eq!(KeyboardBrightness::High.to_string(), "high");
}

#[test]
fn test_keyboard_brightness_default() {
    assert_eq!(KeyboardBrightness::default(), KeyboardBrightness::High);
}

// ------------------------------------------------------------------------
// PowerProfile Tests
// ------------------------------------------------------------------------

#[test]
fn test_power_profile_from_str_valid() {
    assert_eq!(
        PowerProfile::from_str("quiet").unwrap(),
        PowerProfile::Quiet
    );
    assert_eq!(
        PowerProfile::from_str("QUIET").unwrap(),
        PowerProfile::Quiet
    );
    assert_eq!(
        PowerProfile::from_str("balanced").unwrap(),
        PowerProfile::Balanced
    );
    assert_eq!(
        PowerProfile::from_str("Balanced").unwrap(),
        PowerProfile::Balanced
    );
    assert_eq!(
        PowerProfile::from_str("performance").unwrap(),
        PowerProfile::Performance
    );
    assert_eq!(
        PowerProfile::from_str("PERFORMANCE").unwrap(),
        PowerProfile::Performance
    );
}

#[test]
fn test_power_profile_from_str_invalid() {
    assert!(PowerProfile::from_str("turbo").is_err());
    assert!(PowerProfile::from_str("").is_err());
    assert!(PowerProfile::from_str("power-saver").is_err());
}

#[test]
fn test_power_profile_display() {
    assert_eq!(PowerProfile::Quiet.to_string(), "Quiet");
    assert_eq!(PowerProfile::Balanced.to_string(), "Balanced");
    assert_eq!(PowerProfile::Performance.to_string(), "Performance");
}

#[test]
fn test_power_profile_default() {
    assert_eq!(PowerProfile::default(), PowerProfile::Balanced);
}

// ------------------------------------------------------------------------
// AuraMode Tests
// ------------------------------------------------------------------------

#[test]
fn test_aura_mode_from_str_valid() {
    assert_eq!(AuraMode::from_str("static").unwrap(), AuraMode::Static);
    assert_eq!(AuraMode::from_str("Static").unwrap(), AuraMode::Static);
    assert_eq!(AuraMode::from_str("breathe").unwrap(), AuraMode::Breathe);
    assert_eq!(AuraMode::from_str("stars").unwrap(), AuraMode::Stars);
    assert_eq!(AuraMode::from_str("rain").unwrap(), AuraMode::Rain);
    assert_eq!(
        AuraMode::from_str("highlight").unwrap(),
        AuraMode::Highlight
    );
    assert_eq!(AuraMode::from_str("laser").unwrap(), AuraMode::Laser);
    assert_eq!(AuraMode::from_str("ripple").unwrap(), AuraMode::Ripple);
    assert_eq!(AuraMode::from_str("pulse").unwrap(), AuraMode::Pulse);
    assert_eq!(AuraMode::from_str("comet").unwrap(), AuraMode::Comet);
    assert_eq!(AuraMode::from_str("flash").unwrap(), AuraMode::Flash);
}

#[test]
fn test_aura_mode_from_str_rainbow_variants() {
    // Test all accepted formats for rainbow modes
    assert_eq!(
        AuraMode::from_str("rainbow-cycle").unwrap(),
        AuraMode::RainbowCycle
    );
    assert_eq!(
        AuraMode::from_str("rainbowcycle").unwrap(),
        AuraMode::RainbowCycle
    );
    assert_eq!(
        AuraMode::from_str("rainbow cycle").unwrap(),
        AuraMode::RainbowCycle
    );
    assert_eq!(
        AuraMode::from_str("rainbow-wave").unwrap(),
        AuraMode::RainbowWave
    );
    assert_eq!(
        AuraMode::from_str("rainbowwave").unwrap(),
        AuraMode::RainbowWave
    );
    assert_eq!(
        AuraMode::from_str("rainbow wave").unwrap(),
        AuraMode::RainbowWave
    );
}

#[test]
fn test_aura_mode_from_str_invalid() {
    assert!(AuraMode::from_str("invalid").is_err());
    assert!(AuraMode::from_str("").is_err());
    assert!(AuraMode::from_str("strobe").is_err());
}

#[test]
fn test_aura_mode_cli_name() {
    assert_eq!(AuraMode::Static.cli_name(), "static");
    assert_eq!(AuraMode::Breathe.cli_name(), "breathe");
    assert_eq!(AuraMode::RainbowCycle.cli_name(), "rainbow-cycle");
    assert_eq!(AuraMode::RainbowWave.cli_name(), "rainbow-wave");
    assert_eq!(AuraMode::Stars.cli_name(), "stars");
    assert_eq!(AuraMode::Rain.cli_name(), "rain");
    assert_eq!(AuraMode::Highlight.cli_name(), "highlight");
    assert_eq!(AuraMode::Laser.cli_name(), "laser");
    assert_eq!(AuraMode::Ripple.cli_name(), "ripple");
    assert_eq!(AuraMode::Pulse.cli_name(), "pulse");
    assert_eq!(AuraMode::Comet.cli_name(), "comet");
    assert_eq!(AuraMode::Flash.cli_name(), "flash");
}

#[test]
fn test_aura_mode_label() {
    assert_eq!(AuraMode::Static.label(), "Static");
    assert_eq!(AuraMode::RainbowCycle.label(), "Rainbow Cycle");
    assert_eq!(AuraMode::RainbowWave.label(), "Rainbow Wave");
}

#[test]
fn test_aura_mode_needs_colour() {
    // Modes that need colour
    assert!(AuraMode::Static.needs_colour());
    assert!(AuraMode::Breathe.needs_colour());
    assert!(AuraMode::Stars.needs_colour());
    assert!(AuraMode::Highlight.needs_colour());
    assert!(AuraMode::Laser.needs_colour());
    assert!(AuraMode::Ripple.needs_colour());
    assert!(AuraMode::Pulse.needs_colour());
    assert!(AuraMode::Comet.needs_colour());
    assert!(AuraMode::Flash.needs_colour());

    // Modes that don't need colour
    assert!(!AuraMode::RainbowCycle.needs_colour());
    assert!(!AuraMode::RainbowWave.needs_colour());
    assert!(!AuraMode::Rain.needs_colour());
}

#[test]
fn test_aura_mode_needs_colour2() {
    // Only Breathe and Stars need a second colour
    assert!(AuraMode::Breathe.needs_colour2());
    assert!(AuraMode::Stars.needs_colour2());

    // Others don't
    assert!(!AuraMode::Static.needs_colour2());
    assert!(!AuraMode::RainbowCycle.needs_colour2());
    assert!(!AuraMode::Laser.needs_colour2());
}

#[test]
fn test_aura_mode_needs_speed() {
    // Modes that need speed
    assert!(AuraMode::Breathe.needs_speed());
    assert!(AuraMode::RainbowCycle.needs_speed());
    assert!(AuraMode::RainbowWave.needs_speed());
    assert!(AuraMode::Stars.needs_speed());
    assert!(AuraMode::Rain.needs_speed());
    assert!(AuraMode::Highlight.needs_speed());
    assert!(AuraMode::Laser.needs_speed());
    assert!(AuraMode::Ripple.needs_speed());

    // Modes that don't need speed
    assert!(!AuraMode::Static.needs_speed());
    assert!(!AuraMode::Pulse.needs_speed());
    assert!(!AuraMode::Comet.needs_speed());
    assert!(!AuraMode::Flash.needs_speed());
}

#[test]
fn test_aura_mode_needs_direction() {
    // Only RainbowWave needs direction
    assert!(AuraMode::RainbowWave.needs_direction());

    // Others don't
    assert!(!AuraMode::Static.needs_direction());
    assert!(!AuraMode::RainbowCycle.needs_direction());
    assert!(!AuraMode::Breathe.needs_direction());
}

#[test]
fn test_aura_mode_from_dbus_value() {
    assert_eq!(AuraMode::from_dbus_value(0), Some(AuraMode::Static));
    assert_eq!(AuraMode::from_dbus_value(1), Some(AuraMode::Breathe));
    assert_eq!(AuraMode::from_dbus_value(2), Some(AuraMode::RainbowCycle));
    assert_eq!(AuraMode::from_dbus_value(3), Some(AuraMode::RainbowWave));
    assert_eq!(AuraMode::from_dbus_value(4), Some(AuraMode::Stars));
    assert_eq!(AuraMode::from_dbus_value(5), Some(AuraMode::Rain));
    assert_eq!(AuraMode::from_dbus_value(6), Some(AuraMode::Highlight));
    assert_eq!(AuraMode::from_dbus_value(7), Some(AuraMode::Laser));
    assert_eq!(AuraMode::from_dbus_value(8), Some(AuraMode::Ripple));
    assert_eq!(AuraMode::from_dbus_value(9), Some(AuraMode::Pulse));
    assert_eq!(AuraMode::from_dbus_value(10), Some(AuraMode::Comet));
    assert_eq!(AuraMode::from_dbus_value(11), Some(AuraMode::Flash));

    // Invalid values
    assert_eq!(AuraMode::from_dbus_value(12), None);
    assert_eq!(AuraMode::from_dbus_value(100), None);
}

#[test]
fn test_aura_mode_display() {
    assert_eq!(AuraMode::Static.to_string(), "Static");
    assert_eq!(AuraMode::RainbowCycle.to_string(), "Rainbow Cycle");
    assert_eq!(AuraMode::RainbowWave.to_string(), "Rainbow Wave");
}

#[test]
fn test_aura_mode_default() {
    assert_eq!(AuraMode::default(), AuraMode::Static);
}

#[test]
fn test_aura_mode_all_constant() {
    assert_eq!(AuraMode::ALL.len(), 12);
    assert!(AuraMode::ALL.contains(&AuraMode::Static));
    assert!(AuraMode::ALL.contains(&AuraMode::Flash));
}

// ------------------------------------------------------------------------
// AuraSpeed Tests
// ------------------------------------------------------------------------

#[test]
fn test_aura_speed_from_str_valid() {
    assert_eq!(AuraSpeed::from_str("low").unwrap(), AuraSpeed::Low);
    assert_eq!(AuraSpeed::from_str("LOW").unwrap(), AuraSpeed::Low);
    assert_eq!(AuraSpeed::from_str("med").unwrap(), AuraSpeed::Med);
    assert_eq!(AuraSpeed::from_str("Med").unwrap(), AuraSpeed::Med);
    assert_eq!(AuraSpeed::from_str("high").unwrap(), AuraSpeed::High);
    assert_eq!(AuraSpeed::from_str("HIGH").unwrap(), AuraSpeed::High);
}

#[test]
fn test_aura_speed_from_str_invalid() {
    assert!(AuraSpeed::from_str("fast").is_err());
    assert!(AuraSpeed::from_str("").is_err());
    assert!(AuraSpeed::from_str("medium").is_err());
}

#[test]
fn test_aura_speed_display() {
    assert_eq!(AuraSpeed::Low.to_string(), "low");
    assert_eq!(AuraSpeed::Med.to_string(), "med");
    assert_eq!(AuraSpeed::High.to_string(), "high");
}

#[test]
fn test_aura_speed_default() {
    assert_eq!(AuraSpeed::default(), AuraSpeed::Med);
}

// ------------------------------------------------------------------------
// AuraDirection Tests
// ------------------------------------------------------------------------

#[test]
fn test_aura_direction_from_str_valid() {
    assert_eq!(AuraDirection::from_str("up").unwrap(), AuraDirection::Up);
    assert_eq!(AuraDirection::from_str("UP").unwrap(), AuraDirection::Up);
    assert_eq!(
        AuraDirection::from_str("down").unwrap(),
        AuraDirection::Down
    );
    assert_eq!(
        AuraDirection::from_str("Down").unwrap(),
        AuraDirection::Down
    );
    assert_eq!(
        AuraDirection::from_str("left").unwrap(),
        AuraDirection::Left
    );
    assert_eq!(
        AuraDirection::from_str("LEFT").unwrap(),
        AuraDirection::Left
    );
    assert_eq!(
        AuraDirection::from_str("right").unwrap(),
        AuraDirection::Right
    );
    assert_eq!(
        AuraDirection::from_str("Right").unwrap(),
        AuraDirection::Right
    );
}

#[test]
fn test_aura_direction_from_str_invalid() {
    assert!(AuraDirection::from_str("north").is_err());
    assert!(AuraDirection::from_str("").is_err());
}

#[test]
fn test_aura_direction_display() {
    assert_eq!(AuraDirection::Up.to_string(), "up");
    assert_eq!(AuraDirection::Down.to_string(), "down");
    assert_eq!(AuraDirection::Left.to_string(), "left");
    assert_eq!(AuraDirection::Right.to_string(), "right");
}

#[test]
fn test_aura_direction_default() {
    assert_eq!(AuraDirection::default(), AuraDirection::Right);
}

// ------------------------------------------------------------------------
// SlashMode Tests
// ------------------------------------------------------------------------

#[test]
fn test_slash_mode_from_str_valid() {
    assert_eq!(SlashMode::from_str("Static").unwrap(), SlashMode::Static);
    assert_eq!(SlashMode::from_str("Bounce").unwrap(), SlashMode::Bounce);
    assert_eq!(SlashMode::from_str("Slash").unwrap(), SlashMode::Slash);
    assert_eq!(SlashMode::from_str("Loading").unwrap(), SlashMode::Loading);
    assert_eq!(
        SlashMode::from_str("BitStream").unwrap(),
        SlashMode::BitStream
    );
    assert_eq!(
        SlashMode::from_str("Transmission").unwrap(),
        SlashMode::Transmission
    );
    assert_eq!(SlashMode::from_str("Flow").unwrap(), SlashMode::Flow);
    assert_eq!(SlashMode::from_str("Flux").unwrap(), SlashMode::Flux);
    assert_eq!(SlashMode::from_str("Phantom").unwrap(), SlashMode::Phantom);
    assert_eq!(
        SlashMode::from_str("Spectrum").unwrap(),
        SlashMode::Spectrum
    );
    assert_eq!(SlashMode::from_str("Hazard").unwrap(), SlashMode::Hazard);
    assert_eq!(
        SlashMode::from_str("Interfacing").unwrap(),
        SlashMode::Interfacing
    );
    assert_eq!(SlashMode::from_str("Ramp").unwrap(), SlashMode::Ramp);
    assert_eq!(
        SlashMode::from_str("GameOver").unwrap(),
        SlashMode::GameOver
    );
    assert_eq!(SlashMode::from_str("Start").unwrap(), SlashMode::Start);
    assert_eq!(SlashMode::from_str("Buzzer").unwrap(), SlashMode::Buzzer);
}

#[test]
fn test_slash_mode_from_str_invalid() {
    assert!(SlashMode::from_str("invalid").is_err());
    assert!(SlashMode::from_str("").is_err());
    // Note: SlashMode uses case-sensitive matching unlike other enums
    assert!(SlashMode::from_str("static").is_err());
    assert!(SlashMode::from_str("STATIC").is_err());
}

#[test]
fn test_slash_mode_display() {
    assert_eq!(SlashMode::Static.to_string(), "Static");
    assert_eq!(SlashMode::BitStream.to_string(), "BitStream");
    assert_eq!(SlashMode::GameOver.to_string(), "GameOver");
}

#[test]
fn test_slash_mode_default() {
    assert_eq!(SlashMode::default(), SlashMode::Flow);
}

// ------------------------------------------------------------------------
// ProfileState Tests
// ------------------------------------------------------------------------

#[test]
fn test_profile_state_default() {
    let state = ProfileState::default();
    assert_eq!(state.active, PowerProfile::Balanced);
    assert_eq!(state.on_ac, PowerProfile::Balanced);
    assert_eq!(state.on_battery, PowerProfile::Balanced);
}

// ------------------------------------------------------------------------
// AuraModeData Tests
// ------------------------------------------------------------------------

#[test]
fn test_aura_mode_data_default() {
    let data = AuraModeData::default();
    assert_eq!(data.mode, AuraMode::Static);
    assert_eq!(data.zone, 0);
    assert_eq!(data.color1, (0, 0, 0));
    assert_eq!(data.color2, (0, 0, 0));
    assert_eq!(data.speed, AuraSpeed::Med);
    assert_eq!(data.direction, AuraDirection::Right);
}

// ------------------------------------------------------------------------
// SupportedFeatures Tests
// ------------------------------------------------------------------------

#[test]
fn test_supported_features_default() {
    let features = SupportedFeatures::default();
    assert!(!features.asusctl_installed);
    assert!(!features.asusd_running);
    assert!(!features.has_aura);
    assert!(!features.has_platform);
    assert!(!features.has_fan_curves);
    assert!(!features.has_slash);
    assert!(features.keyboard_brightness_levels.is_empty());
    assert!(features.aura_modes.is_empty());
    assert!(features.aura_zones.is_empty());
    assert!(!features.has_charge_control);
    assert!(!features.has_throttle_policy);
    assert!(!features.has_armoury);
}

// ------------------------------------------------------------------------
// SystemInfo Tests
// ------------------------------------------------------------------------

#[test]
fn test_system_info_default() {
    let info = SystemInfo::default();
    assert!(info.asusctl_version.is_empty());
    assert!(info.product_family.is_empty());
    assert!(info.board_name.is_empty());
}
