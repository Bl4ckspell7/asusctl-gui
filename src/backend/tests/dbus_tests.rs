use super::*;
use crate::backend::error::AsusctlError;

#[test]
fn missing_binary_empty_output_is_not_installed() {
    // flatpak-spawn exits non-zero (127) with empty stdout when the host
    // asusctl is absent — the bug this guards against.
    let result = interpret_asusctl_output(false, String::new(), "");
    assert!(matches!(result, Err(AsusctlError::NotInstalled)));
}

#[test]
fn whitespace_only_output_with_failure_is_not_installed() {
    let result = interpret_asusctl_output(false, "   \n".to_string(), "");
    assert!(matches!(result, Err(AsusctlError::NotInstalled)));
}

#[test]
fn asusd_unreachable_is_service_not_running() {
    let refused = interpret_asusctl_output(false, String::new(), "Error: Connection refused");
    assert!(matches!(refused, Err(AsusctlError::ServiceNotRunning)));

    let no_asusd = interpret_asusctl_output(false, String::new(), "failed to connect to asusd");
    assert!(matches!(no_asusd, Err(AsusctlError::ServiceNotRunning)));
}

#[test]
fn service_error_takes_precedence_over_empty_output() {
    // Empty stdout + failure, but the stderr shows asusd is down: asusctl IS
    // installed, the service just isn't running.
    let result = interpret_asusctl_output(false, String::new(), "asusd: Connection refused");
    assert!(matches!(result, Err(AsusctlError::ServiceNotRunning)));
}

#[test]
fn nonzero_exit_with_output_is_ok() {
    // asusctl frequently exits non-zero while still printing useful output.
    let result = interpret_asusctl_output(false, "Product family: ROG".to_string(), "");
    assert_eq!(result.unwrap(), "Product family: ROG");
}

#[test]
fn success_returns_stdout() {
    let result = interpret_asusctl_output(true, "ok".to_string(), "");
    assert_eq!(result.unwrap(), "ok");
}

#[test]
fn success_with_asusd_in_stderr_returns_stdout() {
    // A successful run is never reclassified, however chatty its stderr.
    let result = interpret_asusctl_output(
        true,
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
    let result = interpret_asusctl_output(false, String::new(), stderr);
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
