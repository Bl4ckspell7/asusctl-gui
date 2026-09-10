use super::*;

#[test]
fn test_slash_mode_from_dbus_value() {
    assert_eq!(SlashMode::from_dbus_value(0), Some(SlashMode::Static));
    assert_eq!(SlashMode::from_dbus_value(6), Some(SlashMode::Flow));
    assert_eq!(SlashMode::from_dbus_value(15), Some(SlashMode::Buzzer));
    assert_eq!(SlashMode::from_dbus_value(16), None);
    assert_eq!(SlashMode::from_dbus_value(255), None);
}

#[test]
fn test_slash_set_args_for_enable_and_disable() {
    assert_eq!(slash_set_args(&["--enable"]), ["slash", "set", "--enable"]);
    assert_eq!(
        slash_set_args(&["--disable"]),
        ["slash", "set", "--disable"]
    );
}

#[test]
fn test_slash_set_args_for_numeric_options() {
    let brightness = 128_u8.to_string();
    let interval = 5_u8.to_string();

    assert_eq!(
        slash_set_args(&["--brightness", &brightness]),
        ["slash", "set", "--brightness", "128"]
    );
    assert_eq!(
        slash_set_args(&["--interval", &interval]),
        ["slash", "set", "--interval", "5"]
    );
}

#[test]
fn test_slash_set_args_for_mode() {
    let mode = SlashMode::GameOver.to_string();

    assert_eq!(
        slash_set_args(&["--mode", &mode]),
        ["slash", "set", "--mode", "GameOver"]
    );
}

#[test]
fn test_slash_set_args_for_boolean_event_options() {
    let cases = [
        ("--show-on-boot", true, "true"),
        ("--show-on-shutdown", false, "false"),
        ("--show-on-sleep", true, "true"),
        ("--show-on-battery", false, "false"),
        ("--show-battery-warning", true, "true"),
    ];

    for (option, value, expected_value) in cases {
        assert_eq!(
            slash_set_args(&[option, bool_arg(value)]),
            ["slash", "set", option, expected_value]
        );
    }
}
