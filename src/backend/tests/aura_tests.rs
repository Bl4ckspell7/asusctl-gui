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

// ------------------------------------------------------------------------
// parse_led_power
// ------------------------------------------------------------------------

fn power_state(
    zone: AuraPowerZone,
    boot: bool,
    awake: bool,
    sleep: bool,
    shutdown: bool,
) -> AuraPowerState {
    AuraPowerState {
        zone,
        boot,
        awake,
        sleep,
        shutdown,
    }
}

#[test]
fn test_parse_led_power_single_keyboard_zone() {
    assert_eq!(
        parse_led_power("(a(ubbbb)) 1 1 true true true true").unwrap(),
        vec![power_state(AuraPowerZone::Keyboard, true, true, true, true)]
    );
}

#[test]
fn test_parse_led_power_mixed_states() {
    assert_eq!(
        parse_led_power("(a(ubbbb)) 1 1 true false true true").unwrap(),
        vec![power_state(
            AuraPowerZone::Keyboard,
            true,
            false,
            true,
            true
        )]
    );
}

#[test]
fn test_parse_led_power_multiple_zones() {
    assert_eq!(
        parse_led_power("(a(ubbbb)) 2 1 true true false true 4 false true true false").unwrap(),
        vec![
            power_state(AuraPowerZone::Keyboard, true, true, false, true),
            power_state(AuraPowerZone::RearGlow, false, true, true, false),
        ]
    );
}

#[test]
fn test_parse_led_power_skips_unknown_zones() {
    assert_eq!(
        parse_led_power("(a(ubbbb)) 2 5 true true true true 0 true false false false").unwrap(),
        vec![power_state(AuraPowerZone::Logo, true, false, false, false)]
    );
}

#[test]
fn test_parse_led_power_no_zones() {
    assert!(parse_led_power("(a(ubbbb)) 0").unwrap().is_empty());
}

#[test]
fn test_parse_led_power_rejects_malformed_output() {
    for output in [
        "",
        "(uu(yyy)(yyy)ss) 1 1 true true true true",
        "(a(ubbbb)) 2 1 true true true true",
        "(a(ubbbb)) 1 1 true true true",
        "(a(ubbbb)) 1 1 yes true true true",
        "(a(ubbbb)) 1 keyboard true true true true",
        "(a(ubbbb)) many",
    ] {
        assert!(
            matches!(parse_led_power(output), Err(AsusctlError::ParseError(_))),
            "expected parse error for: {output}"
        );
    }
}

// ------------------------------------------------------------------------
// aura_power_args
// ------------------------------------------------------------------------

#[test]
fn test_aura_power_args_all_on() {
    assert_eq!(
        aura_power_args(&power_state(
            AuraPowerZone::Keyboard,
            true,
            true,
            true,
            true
        )),
        vec![
            "aura",
            "power",
            "keyboard",
            "--boot",
            "--awake",
            "--sleep",
            "--shutdown"
        ]
    );
}

#[test]
fn test_aura_power_args_all_off_has_no_flags() {
    assert_eq!(
        aura_power_args(&power_state(
            AuraPowerZone::Keyboard,
            false,
            false,
            false,
            false
        )),
        vec!["aura", "power", "keyboard"]
    );
}

#[test]
fn test_aura_power_args_single_flag_and_zone_name() {
    assert_eq!(
        aura_power_args(&power_state(
            AuraPowerZone::RearGlow,
            false,
            false,
            true,
            false
        )),
        vec!["aura", "power", "rear-glow", "--sleep"]
    );
}
