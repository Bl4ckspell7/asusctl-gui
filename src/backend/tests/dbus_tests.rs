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
