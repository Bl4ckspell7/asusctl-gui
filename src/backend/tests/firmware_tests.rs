use super::*;

#[test]
fn boot_sound_values_map_to_off_and_on() {
    assert!(!boot_sound_from_value(0).unwrap());
    assert!(boot_sound_from_value(1).unwrap());
}

#[test]
fn unknown_boot_sound_values_are_parse_errors() {
    for value in [-1, 2, 255] {
        assert!(
            matches!(
                boot_sound_from_value(value),
                Err(AsusctlError::ParseError(_))
            ),
            "expected parse error for: {value}"
        );
    }
}
