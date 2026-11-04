//! D-Bus communication helpers and path discovery.

use std::process::Command;
use std::sync::OnceLock;

use super::error::{AsusctlError, Result};

// D-Bus constants
pub const DBUS_DEST: &str = "xyz.ljones.Asusd";
pub const PLATFORM_PATH: &str = "/xyz/ljones";
pub const PLATFORM_INTERFACE: &str = "xyz.ljones.Platform";
pub const AURA_BASE_PATH: &str = "/xyz/ljones/aura";
pub const AURA_INTERFACE: &str = "xyz.ljones.Aura";
pub const SLASH_INTERFACE: &str = "xyz.ljones.Slash";
pub const ARMOURY_BASE_PATH: &str = "/xyz/ljones/asus_armoury";
pub const ARMOURY_INTERFACE: &str = "xyz.ljones.AsusArmoury";

// Cached D-Bus paths (discovered at runtime)
static AURA_PATH: OnceLock<Option<String>> = OnceLock::new();
static SLASH_PATH: OnceLock<Option<String>> = OnceLock::new();
static IN_FLATPAK: OnceLock<bool> = OnceLock::new();

// ============================================================================
// Flatpak Support
// ============================================================================

/// Check if we're running inside a Flatpak sandbox.
pub fn is_flatpak() -> bool {
    *IN_FLATPAK.get_or_init(|| std::path::Path::new("/.flatpak-info").exists())
}

/// Create a Command that runs on the host, using `flatpak-spawn --host` if in Flatpak.
pub fn host_command(program: &str) -> Command {
    if is_flatpak() {
        let mut cmd = Command::new("flatpak-spawn");
        cmd.args(["--host", program]);
        cmd
    } else {
        Command::new(program)
    }
}

// ============================================================================
// Command Execution
// ============================================================================

/// Execute an asusctl command and return its stdout.
pub fn run_asusctl(args: &[&str]) -> Result<String> {
    let output = host_command("asusctl").args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AsusctlError::NotInstalled
        } else {
            AsusctlError::CommandFailed(e.to_string())
        }
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    interpret_asusctl_output(
        output.status.success(),
        output.status.code(),
        stdout,
        &stderr,
    )
}

/// Classify an asusctl invocation's result. Separated from process spawning so
/// the logic is unit-testable.
///
/// - `Error: <reason>` or a version-mismatch notice on stdout -> failure, even
///   with exit status 0: asusctl reports its own errors that way and still
///   exits 0. See [`stdout_reported_error`]. The reason maps to
///   `ServiceNotRunning` when it shows the daemon is unreachable, otherwise to
///   `CommandFailed`.
/// - failure whose stderr shows `asusd` is unreachable -> `ServiceNotRunning`
///   (asusctl is present but the daemon is down). See
///   [`stderr_reports_service_down`] — a bare mention of the daemon does not
///   qualify, and stderr never turns a successful run into a failure.
/// - missing host `asusctl` -> `NotInstalled`. See [`host_binary_missing`].
/// - non-zero exit with useful stdout -> stdout as-is, because some asusctl
///   queries return useful output despite their exit status.
/// - any other failure -> `CommandFailed`, preserving trimmed stderr or an
///   exit-status fallback when stderr is empty.
fn interpret_asusctl_output(
    success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: &str,
) -> Result<String> {
    if let Some(reason) = stdout_reported_error(&stdout) {
        if stderr_reports_service_down(&reason) {
            return Err(AsusctlError::ServiceNotRunning);
        }
        return Err(AsusctlError::CommandFailed(reason));
    }

    if success {
        return Ok(stdout);
    }

    if stderr_reports_service_down(stderr) {
        return Err(AsusctlError::ServiceNotRunning);
    }

    if host_binary_missing(exit_code, &stdout, stderr) {
        return Err(AsusctlError::NotInstalled);
    }

    let stderr = stderr.trim();

    if !stdout.trim().is_empty() {
        return Ok(stdout);
    }

    let message = if !stderr.is_empty() {
        stderr.to_string()
    } else if let Some(code) = exit_code {
        format!("asusctl exited with status {code}")
    } else {
        "asusctl terminated without an exit code".to_string()
    };

    Err(AsusctlError::CommandFailed(message))
}

/// The failure asusctl printed to stdout, if any.
///
/// asusctl exits 0 even when a command fails: it prints `Error: <reason>`
/// followed by an info dump, or a version-mismatch notice and does nothing.
fn stdout_reported_error(stdout: &str) -> Option<String> {
    stdout.lines().map(str::trim).find_map(|line| {
        if let Some(reason) = line.strip_prefix("Error:") {
            Some(reason.trim().to_string())
        } else if line.starts_with("Version mismatch:") {
            Some(line.to_string())
        } else {
            None
        }
    })
}

/// True when a failed host command could not run because its program is missing.
///
/// `flatpak-spawn --host` either exits 127 with empty stdout or reports that it
/// failed to execute the child process because the file does not exist.
pub fn host_binary_missing(exit_code: Option<i32>, stdout: &str, stderr: &str) -> bool {
    let stderr_lowercase = stderr.to_lowercase();
    let flatpak_missing_host_binary = stderr_lowercase.contains("failed to execute child process")
        && stderr_lowercase.contains("no such file or directory");

    (exit_code == Some(127) && stdout.trim().is_empty()) || flatpak_missing_host_binary
}

/// Messages that mean the daemon is unreachable, whoever produced them.
///
/// The two bus implementations word an unowned name differently: dbus-daemon
/// says "was not provided by any .service files", dbus-broker (the Arch/Fedora
/// default) says "is not activatable". The last two entries are asusctl's own
/// prose, which it prints instead of the raw D-Bus error.
const SERVICE_DOWN_MARKERS: &[&str] = &[
    "connection refused",
    "was not provided by any .service files",
    "is not activatable",
    "was not owned by anyone",
    "org.freedesktop.dbus.error.serviceunknown",
    "name has no owner",
    "failed to connect to bus",
    "could not get asusd version",
    "asusd.service running",
];

/// Failure verbs that only mean "service down" when paired with `asusd`.
const CONNECT_FAILURE_VERBS: &[&str] = &[
    "failed to connect",
    "could not connect",
    "unable to connect",
    "cannot connect",
    "not running",
];

/// True when stderr shows `asusd` itself is unreachable, as opposed to a
/// request the live daemon rejected.
///
/// A bare mention of "asusd" is not enough — warnings routinely name the daemon
/// while it is running fine. Likewise `No such property` / `no such interface`
/// come back from a *live* service that simply lacks that member, so they are
/// deliberately not matched here.
fn stderr_reports_service_down(stderr: &str) -> bool {
    let lowered = stderr.to_lowercase();

    if SERVICE_DOWN_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        return true;
    }

    lowered.contains("asusd")
        && CONNECT_FAILURE_VERBS
            .iter()
            .any(|verb| lowered.contains(verb))
}

// ============================================================================
// D-Bus Property Reading
// ============================================================================

/// Read a D-Bus property using busctl.
pub fn read_dbus_property_at(path: &str, interface: &str, property: &str) -> Result<String> {
    busctl_get_properties(path, interface, &[property])
}

/// Read several D-Bus properties with a single busctl call, one value per property.
pub fn read_dbus_properties_at(
    path: &str,
    interface: &str,
    properties: &[&str],
) -> Result<Vec<String>> {
    let output = busctl_get_properties(path, interface, properties)?;
    split_property_values(&output, properties.len())
}

/// Run `busctl get-property` and return its trimmed stdout.
fn busctl_get_properties(path: &str, interface: &str, properties: &[&str]) -> Result<String> {
    let output = host_command("busctl")
        .args(["--system", "get-property", DBUS_DEST, path, interface])
        .args(properties)
        .output()
        .map_err(|e| AsusctlError::CommandFailed(format!("busctl failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr_reports_service_down(&stderr) {
            return Err(AsusctlError::ServiceNotRunning);
        }
        return Err(AsusctlError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Split multi-property `busctl get-property` output into one value per line.
fn split_property_values(output: &str, expected: usize) -> Result<Vec<String>> {
    let values: Vec<String> = output.lines().map(|line| line.trim().to_string()).collect();

    if values.len() != expected {
        return Err(AsusctlError::ParseError(format!(
            "Expected {expected} property values, got {}: {output}",
            values.len()
        )));
    }

    Ok(values)
}

/// Call a D-Bus method (no arguments) and return its stdout.
pub fn call_dbus_method(path: &str, interface: &str, method: &str) -> Result<String> {
    let output = host_command("busctl")
        .args(["--system", "call", DBUS_DEST, path, interface, method])
        .output()
        .map_err(|e| AsusctlError::CommandFailed(format!("busctl call failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AsusctlError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ============================================================================
// D-Bus Value Parsing
// ============================================================================

/// Parse a D-Bus boolean value (format: "b true" or "b false").
pub fn parse_dbus_bool(output: &str) -> Result<bool> {
    let value = output
        .strip_prefix("b ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected boolean, got: {output}")))?;

    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(AsusctlError::ParseError(format!(
            "Invalid boolean value: {value}"
        ))),
    }
}

/// Parse a D-Bus byte value (format: "y 123").
pub fn parse_dbus_byte(output: &str) -> Result<u8> {
    let value = output
        .strip_prefix("y ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected byte, got: {output}")))?;

    value
        .parse()
        .map_err(|_| AsusctlError::ParseError(format!("Invalid byte value: {value}")))
}

/// Parse a D-Bus uint value (format: "u 123").
pub fn parse_dbus_uint(output: &str) -> Result<u32> {
    let value = output
        .strip_prefix("u ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected uint, got: {output}")))?;

    value
        .parse()
        .map_err(|_| AsusctlError::ParseError(format!("Invalid uint value: {value}")))
}

/// Parse a D-Bus int value (format: "i -1").
pub fn parse_dbus_int(output: &str) -> Result<i32> {
    let value = output
        .strip_prefix("i ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected int, got: {output}")))?;

    value
        .parse()
        .map_err(|_| AsusctlError::ParseError(format!("Invalid int value: {value}")))
}

// ============================================================================
// D-Bus Path Discovery
// ============================================================================

/// Discover child paths under /xyz/ljones/aura using busctl.
fn discover_aura_children() -> Result<Vec<String>> {
    let output = host_command("busctl")
        .args(["--system", "tree", "--list", DBUS_DEST])
        .output()
        .map_err(|e| AsusctlError::CommandFailed(format!("busctl tree failed: {e}")))?;

    if !output.status.success() {
        return Err(AsusctlError::ServiceNotRunning);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| line.starts_with(AURA_BASE_PATH) && line.len() > AURA_BASE_PATH.len())
        .map(|s| s.to_string())
        .collect();

    Ok(paths)
}

/// Check if a D-Bus path implements a specific interface by trying to read a known property.
fn path_has_interface(path: &str, interface: &str, test_property: &str) -> bool {
    let output = host_command("busctl")
        .args([
            "--system",
            "get-property",
            DBUS_DEST,
            path,
            interface,
            test_property,
        ])
        .output()
        .ok();

    match output {
        Some(out) => out.status.success(),
        None => false,
    }
}

/// Get the Aura D-Bus path (cached after first discovery).
pub fn get_aura_path() -> Option<&'static String> {
    AURA_PATH
        .get_or_init(|| {
            let paths = discover_aura_children().ok()?;
            // Aura interface has "Brightness" property (keyboard brightness)
            for path in &paths {
                if path_has_interface(path, AURA_INTERFACE, "Brightness") {
                    log::info!("Discovered Aura D-Bus path: {path}");
                    return Some(path.clone());
                }
            }
            log::warn!("No Aura D-Bus path found");
            None
        })
        .as_ref()
}

/// Get the Slash D-Bus path (cached after first discovery).
pub fn get_slash_path() -> Option<&'static String> {
    SLASH_PATH
        .get_or_init(|| {
            let paths = discover_aura_children().ok()?;
            // Slash interface has "Enabled" property
            for path in &paths {
                if path_has_interface(path, SLASH_INTERFACE, "Enabled") {
                    log::info!("Discovered Slash D-Bus path: {path}");
                    return Some(path.clone());
                }
            }
            log::warn!("No Slash D-Bus path found");
            None
        })
        .as_ref()
}

/// Check if the asus-armoury D-Bus interface is available.
///
/// Enumerates paths under `xyz.ljones.Asusd` and checks for any that
/// implement the `xyz.ljones.AsusArmoury` interface, which indicates
/// the asus-armoury kernel driver is loaded.
pub fn has_armoury_interface() -> bool {
    list_armoury_attributes()
        .iter()
        .any(|name| path_has_interface(&armoury_attr_path(name), ARMOURY_INTERFACE, "Name"))
}

/// D-Bus path of a firmware attribute, e.g. `/xyz/ljones/asus_armoury/boot_sound`.
pub fn armoury_attr_path(name: &str) -> String {
    format!("{ARMOURY_BASE_PATH}/{name}")
}

/// List the firmware attributes asusd exposes under `/xyz/ljones/asus_armoury`.
///
/// Returns attribute names such as `dgpu_disable`, or an empty list when the
/// daemon is unreachable.
pub fn list_armoury_attributes() -> Vec<String> {
    let output = host_command("busctl")
        .args(["--system", "tree", "--list", DBUS_DEST])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => {
            parse_armoury_attribute_names(&String::from_utf8_lossy(&out.stdout))
        }
        _ => Vec::new(),
    }
}

/// Extract attribute names from `busctl tree --list` output.
fn parse_armoury_attribute_names(tree: &str) -> Vec<String> {
    let armoury_prefix = format!("{ARMOURY_BASE_PATH}/");
    tree.lines()
        .filter_map(|line| line.trim().strip_prefix(&armoury_prefix))
        .filter(|name| !name.is_empty() && !name.contains('/'))
        .map(|name| name.to_string())
        .collect()
}

#[cfg(test)]
#[path = "tests/dbus_tests.rs"]
mod tests;
