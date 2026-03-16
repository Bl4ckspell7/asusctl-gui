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
