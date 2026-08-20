use gtk4::prelude::*;

use crate::backend::AsusctlError;

use super::AsusctlGuiWindow;

const NOT_INSTALLED_TITLE: &str = "Couldn’t apply the change because asusctl is not installed";
const SERVICE_NOT_RUNNING_TITLE: &str = "Couldn’t apply the change because asusd is not running";

pub(crate) fn show_backend_error(
    source: &impl IsA<gtk4::Widget>,
    fallback_title: &str,
    error: &AsusctlError,
) {
    let Some(root) = source.as_ref().root() else {
        return;
    };
    let Ok(window) = root.downcast::<AsusctlGuiWindow>() else {
        return;
    };

    window.show_error_toast(backend_error_title(error, fallback_title));
}

fn backend_error_title<'a>(error: &AsusctlError, fallback_title: &'a str) -> &'a str {
    match error {
        AsusctlError::NotInstalled => NOT_INSTALLED_TITLE,
        AsusctlError::ServiceNotRunning => SERVICE_NOT_RUNNING_TITLE,
        AsusctlError::CommandFailed(_) | AsusctlError::ParseError(_) => fallback_title,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FALLBACK: &str = "Couldn’t change the power profile";

    #[test]
    fn not_installed_overrides_fallback() {
        assert_eq!(
            backend_error_title(&AsusctlError::NotInstalled, FALLBACK),
            NOT_INSTALLED_TITLE
        );
    }

    #[test]
    fn service_not_running_overrides_fallback() {
        assert_eq!(
            backend_error_title(&AsusctlError::ServiceNotRunning, FALLBACK),
            SERVICE_NOT_RUNNING_TITLE
        );
    }

    #[test]
    fn command_failure_uses_fallback() {
        let error = AsusctlError::CommandFailed("backend details".to_string());

        assert_eq!(backend_error_title(&error, FALLBACK), FALLBACK);
    }

    #[test]
    fn parse_failure_uses_fallback() {
        let error = AsusctlError::ParseError("backend details".to_string());

        assert_eq!(backend_error_title(&error, FALLBACK), FALLBACK);
    }
}
