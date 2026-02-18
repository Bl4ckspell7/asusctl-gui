//! Aura lighting and keyboard brightness control.

use std::process::Stdio;
use std::str::FromStr;

use super::dbus::{
    AURA_INTERFACE, get_aura_path, host_command, is_flatpak, parse_dbus_uint,
    read_dbus_property_at, run_asusctl,
};
use super::error::{AsusctlError, Result};
use super::types::{AuraDirection, AuraMode, AuraModeData, AuraSpeed, KeyboardBrightness};

// ============================================================================
// Public API - Keyboard Brightness
// ============================================================================

/// Get current keyboard brightness via D-Bus.
pub fn get_keyboard_brightness_dbus() -> Result<KeyboardBrightness> {
    let path = get_aura_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Aura D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, AURA_INTERFACE, "Brightness")?;
    let value = parse_dbus_uint(&output)?;

    match value {
        0 => Ok(KeyboardBrightness::Off),
        1 => Ok(KeyboardBrightness::Low),
        2 => Ok(KeyboardBrightness::Med),
        3 => Ok(KeyboardBrightness::High),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown brightness value: {value}"
        ))),
    }
}

/// Set keyboard brightness level.
pub fn set_keyboard_brightness(level: KeyboardBrightness) -> Result<()> {
    run_asusctl(&["leds", "set", &level.to_string()])?;
    eprintln!("[asusctl-gui] Set keyboard brightness to {level}");
    Ok(())
}

// ============================================================================
// Public API - Aura Mode
// ============================================================================

/// Get full aura mode data (mode, colors, speed, direction) via D-Bus.
///
/// Parses the LedModeData property which has signature (uu(yyy)(yyy)ss):
/// (mode, zone, (r1,g1,b1), (r2,g2,b2), speed, direction)
pub fn get_aura_mode_data_dbus() -> Result<AuraModeData> {
    let path = get_aura_path()
        .ok_or_else(|| AsusctlError::CommandFailed("Aura D-Bus path not found".to_string()))?;
    let output = read_dbus_property_at(path, AURA_INTERFACE, "LedModeData")?;

    // Parse output like: (uu(yyy)(yyy)ss) 1 0 255 0 255 255 0 0 "Med" "Right"
    // After the signature, values are space-separated
    let parts: Vec<&str> = output.split_whitespace().collect();

    // Expected: signature + mode + zone + r1 + g1 + b1 + r2 + g2 + b2 + speed + direction
    if parts.len() < 11 {
        return Err(AsusctlError::ParseError(format!(
            "Invalid LedModeData format: {output}"
        )));
    }

    let mode_val: u32 = parts[1]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid mode value".to_string()))?;
    let zone: u32 = parts[2]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid zone value".to_string()))?;
    let r1: u8 = parts[3]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color1 R".to_string()))?;
    let g1: u8 = parts[4]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color1 G".to_string()))?;
    let b1: u8 = parts[5]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color1 B".to_string()))?;
    let r2: u8 = parts[6]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color2 R".to_string()))?;
    let g2: u8 = parts[7]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color2 G".to_string()))?;
    let b2: u8 = parts[8]
        .parse()
        .map_err(|_| AsusctlError::ParseError("Invalid color2 B".to_string()))?;

    // Speed and direction are quoted strings like "Med" "Right"
    let speed_str = parts[9].trim_matches('"');
    let direction_str = parts[10].trim_matches('"');

    let mode = AuraMode::from_dbus_value(mode_val).unwrap_or_default();
    let speed = AuraSpeed::from_str(speed_str).unwrap_or_default();
    let direction = AuraDirection::from_str(direction_str).unwrap_or_default();

    Ok(AuraModeData {
        mode,
        zone,
        color1: (r1, g1, b1),
        color2: (r2, g2, b2),
        speed,
        direction,
    })
}

/// Set aura lighting mode with the given parameters.
pub fn set_aura_mode(
    mode: AuraMode,
    colour: Option<&str>,
    colour2: Option<&str>,
    speed: Option<AuraSpeed>,
    direction: Option<AuraDirection>,
    zone: Option<&str>,
) -> Result<()> {
    let mode_name = mode.cli_name();
    let speed_str = speed.map(|s| s.to_string());
    let direction_str = direction.map(|d| d.to_string());

    let mut args: Vec<&str> = vec!["aura", "effect", mode_name];

    if mode.needs_colour() {
        let c = colour
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires a colour")))?;
        args.extend_from_slice(&["--colour", c]);
    }
    if mode.needs_colour2() {
        let c2 = colour2
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires colour2")))?;
        args.extend_from_slice(&["--colour2", c2]);
    }
    if mode.needs_direction() {
        let d = direction_str
            .as_deref()
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires direction")))?;
        args.extend_from_slice(&["--direction", d]);
    }
    if mode.needs_speed() {
        let s = speed_str
            .as_deref()
            .ok_or_else(|| AsusctlError::CommandFailed(format!("{mode} requires speed")))?;
        args.extend_from_slice(&["--speed", s]);
    }
    if let Some(z) = zone {
        args.extend_from_slice(&["--zone", z]);
    }

    run_asusctl(&args)?;
    eprintln!("[asusctl-gui] Set aura mode to {mode}");
    Ok(())
}

/// Fetch the help text for a given aura mode from the CLI.
pub fn get_aura_mode_help(mode: AuraMode) -> Option<String> {
    run_asusctl(&["aura", "effect", mode.cli_name(), "--help"])
        .ok()
        .map(|s| s.trim().to_string())
}

// ============================================================================
// Public API - Custom Rainbow Effect
// ============================================================================

const RAINBOW_STEPS: u32 = 240;

/// Subpath for the rainbow PID file relative to $HOME.
const RAINBOW_PID_SUBPATH: &str = ".local/state/asusctl-gui/rainbow.pid";

/// Get the path to the rainbow PID file (local filesystem).
fn rainbow_pid_path() -> std::path::PathBuf {
    let state_dir = std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{home}/.local/state")
    });
    std::path::PathBuf::from(state_dir)
        .join("asusctl-gui")
        .join("rainbow.pid")
}

/// Check whether the rainbow process is currently running.
pub fn is_rainbow_running() -> bool {
    if is_flatpak() {
        // PID file is on the host; check via flatpak-spawn
        let script = format!(
            "P=\"$HOME/{RAINBOW_PID_SUBPATH}\"; \
             [ -f \"$P\" ] && kill -0 \"$(cat \"$P\")\" 2>/dev/null"
        );
        return host_command("bash")
            .args(["-c", &script])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    }

    let pid_path = rainbow_pid_path();
    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return false,
    };

    if std::path::Path::new(&format!("/proc/{pid_str}")).exists() {
        true
    } else {
        // Stale PID file, clean up
        let _ = std::fs::remove_file(&pid_path);
        false
    }
}

/// Stop the rainbow process if it is running.
pub fn stop_rainbow() -> Result<()> {
    if is_flatpak() {
        // PID file is on the host; stop via flatpak-spawn
        let script = format!(
            "P=\"$HOME/{RAINBOW_PID_SUBPATH}\"; \
             [ -f \"$P\" ] && kill \"$(cat \"$P\")\" 2>/dev/null; \
             rm -f \"$P\""
        );
        let _ = host_command("bash").args(["-c", &script]).output();
        eprintln!("[asusctl-gui] Stopped rainbow effect");
        return Ok(());
    }

    let pid_path = rainbow_pid_path();
    let pid_str = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return Ok(()),
    };

    let _ = host_command("kill").arg(&pid_str).output();
    let _ = std::fs::remove_file(&pid_path);
    eprintln!("[asusctl-gui] Stopped rainbow effect");
    Ok(())
}

/// Convert HSV to a hex RGB string.
/// Hue: 0.0-360.0, Saturation: 0.0-1.0, Value: 0.0-1.0
fn hsv_to_hex(h: f64, s: f64, v: f64) -> String {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r + m) * 255.0) as u8;
    let g = ((g + m) * 255.0) as u8;
    let b = ((b + m) * 255.0) as u8;

    format!("{r:02x}{g:02x}{b:02x}")
}

/// Start the rainbow effect as a detached background process.
///
/// Precomputes all hex colors in Rust, then spawns a bash script via
/// `setsid --fork` that cycles through them. The process survives app close.
pub fn start_rainbow(speed: u32) -> Result<()> {
    stop_rainbow()?;

    let delay = 0.5 / (speed.max(1) as f64).powf(3.7);

    // Precompute all rainbow colors
    let colors: Vec<String> = (0..RAINBOW_STEPS)
        .map(|i| {
            let hue = i as f64 * 360.0 / RAINBOW_STEPS as f64;
            hsv_to_hex(hue, 1.0, 1.0)
        })
        .collect();

    let colors_str = colors.join(" ");

    if is_flatpak() {
        // In Flatpak, the script runs on the host via flatpak-spawn --host.
        // Use $HOME expansion (resolved by the host shell) for the PID path.
        let script = format!(
            "P=\"$HOME/{RAINBOW_PID_SUBPATH}\"\n\
             mkdir -p \"$(dirname \"$P\")\"\n\
             trap 'rm -f \"$P\"' EXIT\n\
             echo $$ > \"$P\"\n\
             while true; do\n\
             for c in {colors_str}; do\n\
             asusctl aura effect static --colour \"$c\" 2>/dev/null\n\
             sleep {delay:.6}\n\
             done\n\
             done"
        );

        host_command("setsid")
            .args(["--fork", "bash", "-c", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| AsusctlError::CommandFailed(format!("Failed to start rainbow: {e}")))?;
    } else {
        let pid_path = rainbow_pid_path();
        let pid_dir = pid_path.parent().unwrap();

        std::fs::create_dir_all(pid_dir)
            .map_err(|e| AsusctlError::CommandFailed(format!("Failed to create state dir: {e}")))?;

        let pid_path_str = pid_path.display();
        let script = format!(
            "P=\"{pid_path_str}\"\n\
             trap 'rm -f \"$P\"' EXIT\n\
             echo $$ > \"$P\"\n\
             while true; do\n\
             for c in {colors_str}; do\n\
             asusctl aura effect static --colour \"$c\" 2>/dev/null\n\
             sleep {delay:.6}\n\
             done\n\
             done"
        );

        host_command("setsid")
            .args(["--fork", "bash", "-c", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| AsusctlError::CommandFailed(format!("Failed to start rainbow: {e}")))?;
    }

    eprintln!("[asusctl-gui] Started rainbow effect (speed={speed}, delay={delay:.6})");
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
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
}
