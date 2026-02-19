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
// hsv_to_hex Tests
// ------------------------------------------------------------------------

#[test]
fn test_hsv_to_hex_primary_colors() {
    // Red: H=0, S=1, V=1
    assert_eq!(hsv_to_hex(0.0, 1.0, 1.0), "ff0000");

    // Green: H=120, S=1, V=1
    assert_eq!(hsv_to_hex(120.0, 1.0, 1.0), "00ff00");

    // Blue: H=240, S=1, V=1
    assert_eq!(hsv_to_hex(240.0, 1.0, 1.0), "0000ff");
}

#[test]
fn test_hsv_to_hex_secondary_colors() {
    // Yellow: H=60, S=1, V=1
    assert_eq!(hsv_to_hex(60.0, 1.0, 1.0), "ffff00");

    // Cyan: H=180, S=1, V=1
    assert_eq!(hsv_to_hex(180.0, 1.0, 1.0), "00ffff");

    // Magenta: H=300, S=1, V=1
    assert_eq!(hsv_to_hex(300.0, 1.0, 1.0), "ff00ff");
}

#[test]
fn test_hsv_to_hex_white_and_black() {
    // White: S=0, V=1 (hue doesn't matter)
    assert_eq!(hsv_to_hex(0.0, 0.0, 1.0), "ffffff");

    // Black: V=0 (hue and saturation don't matter)
    assert_eq!(hsv_to_hex(0.0, 1.0, 0.0), "000000");
    assert_eq!(hsv_to_hex(180.0, 0.5, 0.0), "000000");
}

#[test]
fn test_hsv_to_hex_grayscale() {
    // 50% gray: S=0, V=0.5
    assert_eq!(hsv_to_hex(0.0, 0.0, 0.5), "7f7f7f");

    // 25% gray
    assert_eq!(hsv_to_hex(0.0, 0.0, 0.25), "3f3f3f");
}

#[test]
fn test_hsv_to_hex_desaturated_colors() {
    // Light red (pink-ish): H=0, S=0.5, V=1
    assert_eq!(hsv_to_hex(0.0, 0.5, 1.0), "ff7f7f");

    // Light green: H=120, S=0.5, V=1
    assert_eq!(hsv_to_hex(120.0, 0.5, 1.0), "7fff7f");
}

#[test]
fn test_hsv_to_hex_darkened_colors() {
    // Dark red: H=0, S=1, V=0.5
    assert_eq!(hsv_to_hex(0.0, 1.0, 0.5), "7f0000");

    // Dark blue: H=240, S=1, V=0.5
    assert_eq!(hsv_to_hex(240.0, 1.0, 0.5), "00007f");
}

#[test]
fn test_hsv_to_hex_intermediate_hues() {
    // Orange: H=30
    assert_eq!(hsv_to_hex(30.0, 1.0, 1.0), "ff7f00");

    // Lime: H=90
    assert_eq!(hsv_to_hex(90.0, 1.0, 1.0), "7fff00");

    // Spring green: H=150
    assert_eq!(hsv_to_hex(150.0, 1.0, 1.0), "00ff7f");

    // Azure: H=210
    assert_eq!(hsv_to_hex(210.0, 1.0, 1.0), "007fff");

    // Violet: H=270
    assert_eq!(hsv_to_hex(270.0, 1.0, 1.0), "7f00ff");

    // Rose: H=330
    assert_eq!(hsv_to_hex(330.0, 1.0, 1.0), "ff007f");
}

#[test]
fn test_hsv_to_hex_edge_cases() {
    // Hue at 360 should be same as 0 (red)
    // Note: Due to modulo operation, h=360 behaves like h=0
    assert_eq!(hsv_to_hex(360.0, 1.0, 1.0), "ff0000");

    // Just before 60 degrees
    let result = hsv_to_hex(59.9, 1.0, 1.0);
    assert!(result.starts_with("ff")); // Should be mostly yellow
}

// ------------------------------------------------------------------------
// Rainbow Effect Tests
// ------------------------------------------------------------------------

#[test]
fn test_rainbow_pid_path_format() {
    let path = rainbow_pid_path();
    let path_str = path.to_string_lossy();

    // Should end with the expected file name
    assert!(path_str.ends_with("asusctl-gui/rainbow.pid"));
}

#[test]
fn test_rainbow_delay_calculation() {
    // Test the delay formula: 0.5 / speed^3.7
    // Speed 1: delay = 0.5 / 1 = 0.5
    let delay_1 = 0.5 / (1u32 as f64).powf(3.7);
    assert!((delay_1 - 0.5).abs() < 0.001);

    // Speed 2: delay = 0.5 / 2^3.7 ≈ 0.038
    let delay_2 = 0.5 / (2u32 as f64).powf(3.7);
    assert!(delay_2 < delay_1);
    assert!(delay_2 > 0.0);

    // Speed 3: delay should be even smaller
    let delay_3 = 0.5 / (3u32 as f64).powf(3.7);
    assert!(delay_3 < delay_2);
}
