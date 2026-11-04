//! GPU mode switching via the asus-armoury `dgpu_disable` and `gpu_mux_mode`
//! firmware attributes.
//!
//! asusd does not write GPU attributes immediately: it queues them in memory
//! and `asus-shutdown.service` applies the queue while the system shuts down.
//! A mode change therefore only takes effect after a reboot, and writing the
//! currently active value clears the queued one.

use super::dbus::{
    ARMOURY_INTERFACE, armoury_attr_path, host_binary_missing, host_command, parse_dbus_int,
    read_dbus_properties_at, run_asusctl,
};
use super::error::{AsusctlError, Result};
use super::shortcut::is_gnome;
use super::system::detect_features;
use super::types::{GpuMode, SupportedFeatures};

const DGPU_DISABLE_ATTR: &str = "dgpu_disable";
const GPU_MUX_MODE_ATTR: &str = "gpu_mux_mode";
const SHUTDOWN_SERVICE: &str = "asus-shutdown.service";

/// Active GPU mode and the mode queued for the next reboot, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GpuState {
    pub active: GpuMode,
    pub pending: Option<GpuMode>,
}

// ============================================================================
// Public API
// ============================================================================

/// GPU modes this laptop supports, in display order.
pub fn available_gpu_modes(features: &SupportedFeatures) -> Vec<GpuMode> {
    GpuMode::ALL
        .into_iter()
        .filter(|mode| match mode {
            GpuMode::Integrated => features.has_dgpu_disable,
            GpuMode::Hybrid => features.has_dgpu_disable || features.has_gpu_mux,
            GpuMode::Ultimate => features.has_gpu_mux,
        })
        .collect()
}

/// Get the active GPU mode and any mode queued for the next reboot via D-Bus.
pub fn get_gpu_state() -> Result<GpuState> {
    let features = detect_features();

    let dgpu_disable = if features.has_dgpu_disable {
        Some(read_attr_values(DGPU_DISABLE_ATTR)?)
    } else {
        None
    };
    let gpu_mux_mode = if features.has_gpu_mux {
        Some(read_attr_values(GPU_MUX_MODE_ATTR)?)
    } else {
        None
    };

    Ok(gpu_state_from_values(dgpu_disable, gpu_mux_mode))
}

/// Queue a GPU mode for the next reboot.
///
/// Writes every GPU attribute this laptop has. Selecting the active mode
/// clears any queued change.
pub fn set_gpu_mode(mode: GpuMode) -> Result<()> {
    let features = detect_features();
    let (dgpu_disable, gpu_mux_mode) = mode.attr_values();

    if features.has_dgpu_disable {
        run_asusctl(&[
            "armoury",
            "set",
            DGPU_DISABLE_ATTR,
            &dgpu_disable.to_string(),
        ])?;
    }
    if features.has_gpu_mux {
        run_asusctl(&[
            "armoury",
            "set",
            GPU_MUX_MODE_ATTR,
            &gpu_mux_mode.to_string(),
        ])?;
    }

    log::info!("Queued GPU mode {mode} for the next reboot");
    Ok(())
}

/// Whether `asus-shutdown.service`, which applies queued GPU changes, is running.
///
/// Only a definite stopped state counts as not running; if systemd can't be
/// queried the service is assumed to be fine.
pub fn is_shutdown_service_active() -> bool {
    match host_command("systemctl")
        .args(["is-active", SHUTDOWN_SERVICE])
        .output()
    {
        Ok(output) => !service_state_is_stopped(String::from_utf8_lossy(&output.stdout).trim()),
        Err(e) => {
            log::warn!("Couldn't query {SHUTDOWN_SERVICE}: {e}");
            true
        }
    }
}

/// Ask the GNOME session to reboot. GNOME shows its own confirmation dialog,
/// which lets apps with unsaved work object.
///
/// Returns `Ok(false)` when not running GNOME or `gnome-session-quit` is
/// missing; the caller must then confirm with the user and use [`reboot_now`].
pub fn request_session_reboot() -> Result<bool> {
    if !is_gnome() {
        return Ok(false);
    }

    if !run_host_command("gnome-session-quit", &["--reboot"])? {
        log::warn!("gnome-session-quit is not available, falling back to systemctl");
        return Ok(false);
    }

    log::info!("Requested reboot via the GNOME session");
    Ok(true)
}

/// Reboot immediately through logind. Callers must confirm with the user first.
pub fn reboot_now() -> Result<()> {
    if !run_host_command("systemctl", &["reboot"])? {
        return Err(AsusctlError::CommandFailed(
            "systemctl is not available".to_string(),
        ));
    }

    log::info!("Requested reboot via systemctl");
    Ok(())
}

// ============================================================================
// Private Helpers
// ============================================================================

/// Read `(CurrentValue, QueuedGpuValue)` of a GPU attribute.
fn read_attr_values(attr: &str) -> Result<(i32, i32)> {
    let values = read_dbus_properties_at(
        &armoury_attr_path(attr),
        ARMOURY_INTERFACE,
        &["CurrentValue", "QueuedGpuValue"],
    )?;
    Ok((parse_dbus_int(&values[0])?, parse_dbus_int(&values[1])?))
}

/// Build the GPU state from `(current, queued)` pairs of each present attribute.
///
/// A queued value of `-1` means nothing is queued for that attribute.
fn gpu_state_from_values(
    dgpu_disable: Option<(i32, i32)>,
    gpu_mux_mode: Option<(i32, i32)>,
) -> GpuState {
    let current = |values: Option<(i32, i32)>| values.map(|(current, _)| current);
    let effective = |values: Option<(i32, i32)>| {
        values.map(|(current, queued)| if queued >= 0 { queued } else { current })
    };

    let active = GpuMode::from_attrs(current(dgpu_disable), current(gpu_mux_mode));
    let target = GpuMode::from_attrs(effective(dgpu_disable), effective(gpu_mux_mode));

    GpuState {
        active,
        pending: (target != active).then_some(target),
    }
}

/// Whether a `systemctl is-active` state means the unit is stopped.
fn service_state_is_stopped(state: &str) -> bool {
    matches!(state, "inactive" | "failed" | "deactivating")
}

/// Run a host command. Returns `Ok(false)` when the program is missing on the host.
fn run_host_command(program: &str, args: &[&str]) -> Result<bool> {
    let output = match host_command(program).args(args).output() {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => {
            return Err(AsusctlError::CommandFailed(format!(
                "{program} failed: {e}"
            )));
        }
    };

    if output.status.success() {
        return Ok(true);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if host_binary_missing(output.status.code(), &stdout, &stderr) {
        return Ok(false);
    }

    let stderr = stderr.trim();
    let message = if stderr.is_empty() {
        format!("{program} {} failed", args.join(" "))
    } else {
        stderr.to_string()
    };
    Err(AsusctlError::CommandFailed(message))
}

#[cfg(test)]
#[path = "tests/gpu_tests.rs"]
mod tests;
