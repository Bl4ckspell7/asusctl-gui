use super::*;

#[test]
fn test_error_display_not_installed() {
    let err = AsusctlError::NotInstalled;
    assert_eq!(err.to_string(), "asusctl is not installed");
}

#[test]
fn test_error_display_service_not_running() {
    let err = AsusctlError::ServiceNotRunning;
    assert_eq!(err.to_string(), "asusd service is not running");
}

#[test]
fn test_error_display_command_failed() {
    let err = AsusctlError::CommandFailed("exit code 1".to_string());
    assert_eq!(err.to_string(), "Command failed: exit code 1");

    let err2 = AsusctlError::CommandFailed("permission denied".to_string());
    assert_eq!(err2.to_string(), "Command failed: permission denied");
}

#[test]
fn test_error_display_parse_error() {
    let err = AsusctlError::ParseError("invalid format".to_string());
    assert_eq!(err.to_string(), "Parse error: invalid format");

    let err2 = AsusctlError::ParseError("unexpected EOF".to_string());
    assert_eq!(err2.to_string(), "Parse error: unexpected EOF");
}

#[test]
fn test_error_debug() {
    let err = AsusctlError::NotInstalled;
    let debug_str = format!("{:?}", err);
    assert_eq!(debug_str, "NotInstalled");

    let err2 = AsusctlError::CommandFailed("test".to_string());
    let debug_str2 = format!("{:?}", err2);
    assert!(debug_str2.contains("CommandFailed"));
    assert!(debug_str2.contains("test"));
}

#[test]
fn test_error_clone() {
    let err = AsusctlError::CommandFailed("original".to_string());
    let cloned = err.clone();
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_result_type_alias() {
    fn test_ok() -> Result<i32> {
        Ok(42)
    }

    fn test_err() -> Result<i32> {
        Err(AsusctlError::NotInstalled)
    }

    assert_eq!(test_ok().unwrap(), 42);
    assert!(test_err().is_err());
}

#[test]
fn test_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(AsusctlError::NotInstalled);
    assert_eq!(err.to_string(), "asusctl is not installed");
}
