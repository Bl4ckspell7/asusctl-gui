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
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check for common error patterns
    if stderr.contains("Connection refused") || stderr.contains("asusd") {
        return Err(AsusctlError::ServiceNotRunning);
    }

    // Note: asusctl often returns non-zero but still provides useful output
    let _ = output.status.success();

    Ok(stdout)
}

// ============================================================================
// D-Bus Property Reading
// ============================================================================

/// Read a D-Bus property using busctl.
pub fn read_dbus_property_at(path: &str, interface: &str, property: &str) -> Result<String> {
    let output = host_command("busctl")
        .args([
            "--system",
            "get-property",
            DBUS_DEST,
            path,
            interface,
            property,
        ])
        .output()
        .map_err(|e| AsusctlError::CommandFailed(format!("busctl failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such") || stderr.contains("not found") {
            return Err(AsusctlError::ServiceNotRunning);
        }
        return Err(AsusctlError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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
                    eprintln!("[asusctl-gui] Discovered Aura D-Bus path: {path}");
                    return Some(path.clone());
                }
            }
            eprintln!("[asusctl-gui] Warning: No Aura D-Bus path found");
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
                    eprintln!("[asusctl-gui] Discovered Slash D-Bus path: {path}");
                    return Some(path.clone());
                }
            }
            eprintln!("[asusctl-gui] Warning: No Slash D-Bus path found");
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
    let output = host_command("busctl")
        .args(["--system", "tree", "--list", DBUS_DEST])
        .output()
        .ok();

    let Some(output) = output else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("/xyz/ljones/asus_armoury/"))
        .collect();

    for path in paths {
        if path_has_interface(path, ARMOURY_INTERFACE, "Name") {
            return true;
        }
    }

    false
}
