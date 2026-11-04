use super::*;
use crate::backend::error::AsusctlError;

#[test]
fn exit_127_with_empty_output_is_not_installed() {
    // Some flatpak-spawn versions report a missing host binary only through
    // exit 127, with no stdout or stderr.
    let result = interpret_asusctl_output(false, Some(127), String::new(), "");
    assert!(matches!(result, Err(AsusctlError::NotInstalled)));
}

#[test]
fn exit_127_with_whitespace_only_output_is_not_installed() {
    let result = interpret_asusctl_output(false, Some(127), "   \n".to_string(), "");
    assert!(matches!(result, Err(AsusctlError::NotInstalled)));
}

#[test]
fn flatpak_missing_host_child_is_not_installed() {
    let stderr = "Portal call failed: Failed to start command: Failed to execute child process \
                  “asusctl” (No such file or directory)";
    let result = interpret_asusctl_output(false, Some(1), String::new(), stderr);
    assert!(matches!(result, Err(AsusctlError::NotInstalled)));
}

#[test]
fn flatpak_missing_host_child_detection_requires_both_markers() {
    for stderr in [
        "Failed to execute child process ‘asusctl’",
        "No such file or directory",
    ] {
        let result = interpret_asusctl_output(false, Some(1), String::new(), stderr);
        assert!(matches!(result, Err(AsusctlError::CommandFailed(message)) if message == stderr));
    }
}

#[test]
fn parser_error_is_command_failed_with_trimmed_stderr() {
    let stderr =
        "  Unrecognized argument: --enable\n\nRun asusctl --help for more information.  \n";
    let result = interpret_asusctl_output(false, Some(1), String::new(), stderr);

    assert!(matches!(
        result,
        Err(AsusctlError::CommandFailed(message))
            if message
                == "Unrecognized argument: --enable\n\nRun asusctl --help for more information."
    ));
}

#[test]
fn empty_stderr_failure_reports_exit_status() {
    let result = interpret_asusctl_output(false, Some(2), String::new(), " \n");
    assert!(matches!(
        result,
        Err(AsusctlError::CommandFailed(message))
            if message == "asusctl exited with status 2"
    ));
}

#[test]
fn signal_failure_without_stderr_reports_missing_exit_code() {
    let result = interpret_asusctl_output(false, None, String::new(), "");
    assert!(matches!(
        result,
        Err(AsusctlError::CommandFailed(message))
            if message == "asusctl terminated without an exit code"
    ));
}

#[test]
fn asusd_unreachable_is_service_not_running() {
    let refused =
        interpret_asusctl_output(false, Some(1), String::new(), "Error: Connection refused");
    assert!(matches!(refused, Err(AsusctlError::ServiceNotRunning)));

    let no_asusd =
        interpret_asusctl_output(false, Some(1), String::new(), "failed to connect to asusd");
    assert!(matches!(no_asusd, Err(AsusctlError::ServiceNotRunning)));
}

#[test]
fn service_error_takes_precedence_over_empty_output() {
    // Empty stdout + failure, but the stderr shows asusd is down: asusctl IS
    // installed, the service just isn't running.
    let result =
        interpret_asusctl_output(false, Some(127), String::new(), "asusd: Connection refused");
    assert!(matches!(result, Err(AsusctlError::ServiceNotRunning)));
}

#[test]
fn nonzero_exit_with_output_is_ok() {
    // asusctl frequently exits non-zero while still printing useful output.
    let result = interpret_asusctl_output(false, Some(1), "Product family: ROG".to_string(), "");
    assert_eq!(result.unwrap(), "Product family: ROG");
}

#[test]
fn exit_127_with_useful_output_is_ok() {
    let result = interpret_asusctl_output(false, Some(127), "Product family: ROG".to_string(), "");
    assert_eq!(result.unwrap(), "Product family: ROG");
}

#[test]
fn success_returns_stdout() {
    let result = interpret_asusctl_output(true, Some(0), "ok".to_string(), "");
    assert_eq!(result.unwrap(), "ok");
}

#[test]
fn error_line_on_stdout_fails_despite_exit_zero() {
    // asusctl prints its own errors to stdout and still exits 0.
    let stdout = "\nError: Firmware attribute 'does_not_exist_attr' not found\n\n\
                  Software version: 6.5.0\n  Product family: ROG Zephyrus G14\n";
    let result = interpret_asusctl_output(true, Some(0), stdout.to_string(), "");
    assert!(matches!(
        result,
        Err(AsusctlError::CommandFailed(message))
            if message == "Firmware attribute 'does_not_exist_attr' not found"
    ));
}

#[test]
fn daemon_unreachable_error_line_is_service_not_running() {
    let stdout = "\nError: org.freedesktop.DBus.Error.ServiceUnknown: The name \
                  xyz.ljones.Asusd was not provided by any .service files\n";
    let result = interpret_asusctl_output(true, Some(0), stdout.to_string(), "");
    assert!(matches!(result, Err(AsusctlError::ServiceNotRunning)));
}

#[test]
fn version_mismatch_on_stdout_fails() {
    let stdout = "Version mismatch: asusctl = 6.5.0, asusd = 6.4.0\n";
    let result = interpret_asusctl_output(true, Some(0), stdout.to_string(), "");
    assert!(matches!(
        result,
        Err(AsusctlError::CommandFailed(message))
            if message == "Version mismatch: asusctl = 6.5.0, asusd = 6.4.0"
    ));
}

#[test]
fn error_word_inside_a_line_is_not_a_failure() {
    let stdout = "Active profile: Quiet\nLast Error: none\n";
    let result = interpret_asusctl_output(true, Some(0), stdout.to_string(), "");
    assert_eq!(result.unwrap(), stdout);
}

#[test]
fn success_with_asusd_in_stderr_returns_stdout() {
    // A successful run is never reclassified, however chatty its stderr.
    let result = interpret_asusctl_output(
        true,
        Some(0),
        "Active profile: Quiet".to_string(),
        "warning: asusd config key 'foo' is deprecated",
    );
    assert_eq!(result.unwrap(), "Active profile: Quiet");
}

#[test]
fn failure_with_asusd_warning_and_output_returns_stdout() {
    // asusd is named but nothing says it is unreachable, and there is real
    // output — this must not read as a dead service.
    let result = interpret_asusctl_output(
        false,
        Some(1),
        "Product family: ROG".to_string(),
        "warning: asusd config key 'foo' is deprecated",
    );
    assert_eq!(result.unwrap(), "Product family: ROG");
}

#[test]
fn transport_failures_are_service_down() {
    for stderr in [
        "Error: Connection refused",
        // dbus-daemon wording for an unowned name...
        "The name xyz.ljones.Asusd was not provided by any .service files",
        // ...and dbus-broker's, which is what Arch and Fedora actually run.
        "Failed to get property Brightness on interface xyz.ljones.Aura: The name is not activatable",
        "org.freedesktop.DBus.Error.ServiceUnknown: no such service",
        "Error: Name has no owner",
        "Failed to connect to bus: No such file or directory",
    ] {
        assert!(
            stderr_reports_service_down(stderr),
            "expected service-down for: {stderr}"
        );
    }
}

#[test]
fn asusctl_own_daemon_down_prose_is_service_down() {
    // Verbatim from asusctl 6.3.8 when asusd.service is stopped. It names no
    // connect verb, so it has to be matched on its own.
    let stderr = "Could not get asusd version: \nIs asusd.service running?";
    assert!(stderr_reports_service_down(stderr));

    // The whole point: this must not be mistaken for a missing binary.
    let result = interpret_asusctl_output(false, Some(1), String::new(), stderr);
    assert!(matches!(result, Err(AsusctlError::ServiceNotRunning)));
}

#[test]
fn asusd_with_failure_verb_is_service_down() {
    for stderr in [
        "failed to connect to asusd",
        "Could not connect to asusd over dbus",
        "Unable to connect to asusd",
        "cannot connect to asusd",
        "asusd is not running",
    ] {
        assert!(
            stderr_reports_service_down(stderr),
            "expected service-down for: {stderr}"
        );
    }
}

#[test]
fn live_service_errors_are_not_service_down() {
    // These all come back from a service that is up but lacks the member, or
    // merely mention the daemon in passing.
    for stderr in [
        "Failed to get property Brightness: No such property",
        "Unknown property or interface",
        "No such interface 'xyz.ljones.Slash'",
        "No such object path '/xyz/ljones/aura/foo'",
        "warning: asusd config key 'foo' is deprecated",
        "asusd version 6.1.4",
    ] {
        assert!(
            !stderr_reports_service_down(stderr),
            "expected NOT service-down for: {stderr}"
        );
    }
}

#[test]
fn parse_dbus_int_accepts_positive_and_negative_values() {
    assert_eq!(parse_dbus_int("i 1").unwrap(), 1);
    assert_eq!(parse_dbus_int("i 87").unwrap(), 87);
    assert_eq!(parse_dbus_int("i -1").unwrap(), -1);
}

#[test]
fn parse_dbus_int_rejects_other_types_and_garbage() {
    for output in ["u 1", "y 1", "1", "i ", "i one", "i 1.5"] {
        assert!(
            matches!(parse_dbus_int(output), Err(AsusctlError::ParseError(_))),
            "expected parse error for: {output}"
        );
    }
}

#[test]
fn armoury_attribute_names_come_from_direct_children() {
    let tree = "/\n\
                /xyz\n\
                /xyz/ljones\n\
                /xyz/ljones/asus_armoury\n\
                /xyz/ljones/asus_armoury/dgpu_disable\n\
                /xyz/ljones/asus_armoury/gpu_mux_mode\n\
                /xyz/ljones/asus_armoury/ppt_pl1_spl\n\
                /xyz/ljones/aura\n\
                /xyz/ljones/aura/19b6_3_4\n";

    assert_eq!(
        parse_armoury_attribute_names(tree),
        vec!["dgpu_disable", "gpu_mux_mode", "ppt_pl1_spl"]
    );
}

#[test]
fn armoury_attribute_names_empty_without_armoury_tree() {
    assert!(parse_armoury_attribute_names("/\n/xyz/ljones\n/xyz/ljones/aura\n").is_empty());
    assert!(parse_armoury_attribute_names("").is_empty());
}

#[test]
fn host_binary_missing_detects_exit_127_and_flatpak_markers() {
    assert!(host_binary_missing(Some(127), "", ""));
    assert!(host_binary_missing(
        Some(1),
        "",
        "Failed to execute child process “gnome-session-quit” (No such file or directory)"
    ));
    assert!(!host_binary_missing(Some(127), "some output", ""));
    assert!(!host_binary_missing(Some(1), "", "Access denied"));
}

#[test]
fn split_property_values_returns_one_value_per_line() {
    assert_eq!(
        split_property_values("i 1\ni -1", 2).unwrap(),
        vec!["i 1", "i -1"]
    );
}

#[test]
fn split_property_values_rejects_wrong_count() {
    assert!(matches!(
        split_property_values("i 1", 2),
        Err(AsusctlError::ParseError(_))
    ));
    assert!(matches!(
        split_property_values("", 1),
        Err(AsusctlError::ParseError(_))
    ));
}

#[test]
fn armoury_attr_path_appends_the_attribute_name() {
    assert_eq!(
        armoury_attr_path("boot_sound"),
        "/xyz/ljones/asus_armoury/boot_sound"
    );
}
