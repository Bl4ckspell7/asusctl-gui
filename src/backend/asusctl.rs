//! Backend module for wrapping asusctl CLI commands.
//!
//! This module provides structured Rust types and functions for interacting
//! with asusctl, handling errors gracefully when asusctl is not installed
//! or the asusd service is not running.
//!
//! State reading strategy:
//! - Platform (profiles, charge limit): D-Bus via xyz.ljones.Platform
//! - Slash: Config file at /etc/asusd/slash.ron (no D-Bus interface available)
//! - Aura/Keyboard brightness: CLI output parsing

use std::fs;
use std::process::Command;
use std::str::FromStr;

// D-Bus constants
const DBUS_DEST: &str = "xyz.ljones.Asusd";
const DBUS_PATH: &str = "/xyz/ljones";
const PLATFORM_INTERFACE: &str = "xyz.ljones.Platform";

// Aura D-Bus (keyboard lighting)
const AURA_PATH: &str = "/xyz/ljones/aura/19b6_4_4";
const AURA_INTERFACE: &str = "xyz.ljones.Aura";

// Slash D-Bus (LED bar) - found under aura path
const SLASH_PATH: &str = "/xyz/ljones/aura/193b_5_5";
const SLASH_INTERFACE: &str = "xyz.ljones.Slash";

// Config file paths (fallback)
const SLASH_CONFIG_PATH: &str = "/etc/asusd/slash.ron";

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
pub enum AsusctlError {
    /// asusctl binary not found
    NotInstalled,
    /// asusd service not running
    ServiceNotRunning,
    /// Command execution failed
    CommandFailed(String),
    /// Failed to parse command output
    ParseError(String),
}

impl std::fmt::Display for AsusctlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled => write!(f, "asusctl is not installed"),
            Self::ServiceNotRunning => write!(f, "asusd service is not running"),
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for AsusctlError {}

pub type Result<T> = std::result::Result<T, AsusctlError>;

// ============================================================================
// Keyboard Brightness
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyboardBrightness {
    Off,
    Low,
    Med,
    #[default]
    High,
}

impl std::fmt::Display for KeyboardBrightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "off"),
            Self::Low => write!(f, "low"),
            Self::Med => write!(f, "med"),
            Self::High => write!(f, "high"),
        }
    }
}

impl FromStr for KeyboardBrightness {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "off" => Ok(Self::Off),
            "low" => Ok(Self::Low),
            "med" => Ok(Self::Med),
            "high" => Ok(Self::High),
            _ => Err(AsusctlError::ParseError(format!(
                "Unknown brightness level: {}",
                s
            ))),
        }
    }
}

// ============================================================================
// Power Profile
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerProfile {
    Quiet,
    #[default]
    Balanced,
    Performance,
}

impl std::fmt::Display for PowerProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quiet => write!(f, "Quiet"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Performance => write!(f, "Performance"),
        }
    }
}

impl FromStr for PowerProfile {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "quiet" => Ok(Self::Quiet),
            "balanced" => Ok(Self::Balanced),
            "performance" => Ok(Self::Performance),
            _ => Err(AsusctlError::ParseError(format!(
                "Unknown power profile: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfileState {
    pub active: PowerProfile,
    pub on_ac: PowerProfile,
    pub on_battery: PowerProfile,
}

// ============================================================================
// Aura Modes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuraMode {
    #[default]
    Static,
    Breathe,
    Pulse,
}

impl std::fmt::Display for AuraMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static => write!(f, "Static"),
            Self::Breathe => write!(f, "Breathe"),
            Self::Pulse => write!(f, "Pulse"),
        }
    }
}

impl FromStr for AuraMode {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "static" => Ok(Self::Static),
            "breathe" => Ok(Self::Breathe),
            "pulse" => Ok(Self::Pulse),
            _ => Err(AsusctlError::ParseError(format!(
                "Unknown aura mode: {}",
                s
            ))),
        }
    }
}

// ============================================================================
// Supported Features (from --show-supported)
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SupportedFeatures {
    pub has_aura: bool,
    pub has_platform: bool,
    pub has_fan_curves: bool,
    pub has_slash: bool,
    pub keyboard_brightness_levels: Vec<KeyboardBrightness>,
    pub aura_modes: Vec<AuraMode>,
    pub has_charge_control: bool,
    pub has_throttle_policy: bool,
}

// ============================================================================
// System Info (from --version)
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub asusctl_version: String,
    pub product_family: String,
    pub board_name: String,
}

// ============================================================================
// Command Execution Helper
// ============================================================================

fn run_asusctl(args: &[&str]) -> Result<String> {
    let output = Command::new("asusctl").args(args).output().map_err(|e| {
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

    if !output.status.success() && !stdout.is_empty() {
        // Some commands output to stdout even on "failure"
        // asusctl often returns non-zero but still provides useful output
    }

    Ok(stdout)
}

// ============================================================================
// Public API Functions
// ============================================================================

/// Check if asusctl is available and asusd service is running
pub fn check_availability() -> Result<()> {
    run_asusctl(&["--version"])?;
    Ok(())
}

/// Get system information (version, product family, board name)
pub fn get_system_info() -> Result<SystemInfo> {
    let output = run_asusctl(&["--version"])?;
    parse_system_info(&output)
}

/// Get supported features for this laptop
pub fn get_supported_features() -> Result<SupportedFeatures> {
    let output = run_asusctl(&["--show-supported"])?;
    parse_supported_features(&output)
}

/// Get current keyboard brightness level
pub fn get_keyboard_brightness() -> Result<KeyboardBrightness> {
    let output = run_asusctl(&["--kbd-bright"])?;
    parse_keyboard_brightness(&output)
}

/// Set keyboard brightness level
pub fn set_keyboard_brightness(level: KeyboardBrightness) -> Result<()> {
    run_asusctl(&["--kbd-bright", &level.to_string()])?;
    Ok(())
}

/// Get current profile state (active, on AC, on battery)
pub fn get_profile_state() -> Result<ProfileState> {
    let output = run_asusctl(&["profile", "--profile-get"])?;
    parse_profile_state(&output)
}

/// Set the active power profile
pub fn set_profile(profile: PowerProfile) -> Result<()> {
    run_asusctl(&["profile", "--profile-set", &profile.to_string()])?;
    Ok(())
}

/// Enable slash LED bar
pub fn enable_slash() -> Result<()> {
    run_asusctl(&["slash", "--enable"])?;
    Ok(())
}

/// Disable slash LED bar
pub fn disable_slash() -> Result<()> {
    run_asusctl(&["slash", "--disable"])?;
    Ok(())
}

/// Set slash brightness (0-255)
pub fn set_slash_brightness(brightness: u8) -> Result<()> {
    run_asusctl(&["slash", "--brightness", &brightness.to_string()])?;
    Ok(())
}

// ============================================================================
// Parsing Functions
// ============================================================================

fn parse_system_info(output: &str) -> Result<SystemInfo> {
    let mut info = SystemInfo::default();

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("asusctl version:") {
            info.asusctl_version = line
                .strip_prefix("asusctl version:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if line.starts_with("Product family:") {
            info.product_family = line
                .strip_prefix("Product family:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if line.starts_with("Board name:") {
            info.board_name = line
                .strip_prefix("Board name:")
                .unwrap_or("")
                .trim()
                .to_string();
        }
    }

    Ok(info)
}

fn parse_supported_features(output: &str) -> Result<SupportedFeatures> {
    let mut features = SupportedFeatures::default();

    // Parse core functions
    if output.contains("xyz.ljones.Aura") {
        features.has_aura = true;
    }
    if output.contains("xyz.ljones.Platform") {
        features.has_platform = true;
    }
    if output.contains("xyz.ljones.FanCurves") {
        features.has_fan_curves = true;
    }
    if output.contains("xyz.ljones.Slash") {
        features.has_slash = true;
    }

    // Parse platform properties
    if output.contains("ChargeControlEndThreshold") {
        features.has_charge_control = true;
    }
    if output.contains("ThrottlePolicy") {
        features.has_throttle_policy = true;
    }

    // Parse keyboard brightness levels
    let brightness_section = extract_section(output, "Supported Keyboard Brightness:");
    for level in ["Off", "Low", "Med", "High"] {
        if brightness_section.contains(level) {
            if let Ok(brightness) = KeyboardBrightness::from_str(level) {
                features.keyboard_brightness_levels.push(brightness);
            }
        }
    }

    // Parse aura modes
    let aura_section = extract_section(output, "Supported Aura Modes:");
    for mode in ["Static", "Breathe", "Pulse"] {
        if aura_section.contains(mode) {
            if let Ok(aura_mode) = AuraMode::from_str(mode) {
                features.aura_modes.push(aura_mode);
            }
        }
    }

    Ok(features)
}

fn parse_keyboard_brightness(output: &str) -> Result<KeyboardBrightness> {
    // Output format: "Current keyboard led brightness: High"
    for line in output.lines() {
        if line.contains("Current keyboard led brightness:") {
            let level = line
                .split(':')
                .nth(1)
                .ok_or_else(|| AsusctlError::ParseError("Missing brightness value".to_string()))?
                .trim();
            return KeyboardBrightness::from_str(level);
        }
    }

    Err(AsusctlError::ParseError(
        "Could not find brightness level in output".to_string(),
    ))
}

fn parse_profile_state(output: &str) -> Result<ProfileState> {
    let mut state = ProfileState::default();

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("Active profile is") {
            let profile = line.strip_prefix("Active profile is").unwrap_or("").trim();
            state.active = PowerProfile::from_str(profile)?;
        } else if line.starts_with("Profile on AC is") {
            let profile = line.strip_prefix("Profile on AC is").unwrap_or("").trim();
            state.on_ac = PowerProfile::from_str(profile)?;
        } else if line.starts_with("Profile on Battery is") {
            let profile = line
                .strip_prefix("Profile on Battery is")
                .unwrap_or("")
                .trim();
            state.on_battery = PowerProfile::from_str(profile)?;
        }
    }

    Ok(state)
}

/// Helper to extract a section from the output (between a header and the next header or end)
fn extract_section(output: &str, header: &str) -> String {
    let mut in_section = false;
    let mut section = String::new();
    let mut bracket_depth = 0;

    for line in output.lines() {
        if line.contains(header) {
            in_section = true;
            continue;
        }

        if in_section {
            // Track bracket depth to know when section ends
            bracket_depth += line.matches('[').count() as i32;
            bracket_depth -= line.matches(']').count() as i32;

            section.push_str(line);
            section.push('\n');

            // Section ends when we close all brackets and hit a new section
            if bracket_depth <= 0 && line.contains(']') {
                break;
            }
        }
    }

    section
}

// ============================================================================
// Slash State
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlashMode {
    Bounce,
    Slash,
    Loading,
    BitStream,
    Transmission,
    #[default]
    Flow,
    Flux,
    Phantom,
    Spectrum,
    Hazard,
    Interfacing,
    Ramp,
    GameOver,
    Start,
    Buzzer,
}

impl std::fmt::Display for SlashMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bounce => write!(f, "Bounce"),
            Self::Slash => write!(f, "Slash"),
            Self::Loading => write!(f, "Loading"),
            Self::BitStream => write!(f, "BitStream"),
            Self::Transmission => write!(f, "Transmission"),
            Self::Flow => write!(f, "Flow"),
            Self::Flux => write!(f, "Flux"),
            Self::Phantom => write!(f, "Phantom"),
            Self::Spectrum => write!(f, "Spectrum"),
            Self::Hazard => write!(f, "Hazard"),
            Self::Interfacing => write!(f, "Interfacing"),
            Self::Ramp => write!(f, "Ramp"),
            Self::GameOver => write!(f, "GameOver"),
            Self::Start => write!(f, "Start"),
            Self::Buzzer => write!(f, "Buzzer"),
        }
    }
}

impl FromStr for SlashMode {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Bounce" => Ok(Self::Bounce),
            "Slash" => Ok(Self::Slash),
            "Loading" => Ok(Self::Loading),
            "BitStream" => Ok(Self::BitStream),
            "Transmission" => Ok(Self::Transmission),
            "Flow" => Ok(Self::Flow),
            "Flux" => Ok(Self::Flux),
            "Phantom" => Ok(Self::Phantom),
            "Spectrum" => Ok(Self::Spectrum),
            "Hazard" => Ok(Self::Hazard),
            "Interfacing" => Ok(Self::Interfacing),
            "Ramp" => Ok(Self::Ramp),
            "GameOver" => Ok(Self::GameOver),
            "Start" => Ok(Self::Start),
            "Buzzer" => Ok(Self::Buzzer),
            _ => Err(AsusctlError::ParseError(format!(
                "Unknown slash mode: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SlashState {
    pub enabled: bool,
    pub brightness: u8,
    pub interval: u8,
    pub mode: SlashMode,
}

// ============================================================================
// D-Bus Helper Functions
// ============================================================================

/// Read a D-Bus property using busctl (from default path /xyz/ljones)
fn read_dbus_property(interface: &str, property: &str) -> Result<String> {
    read_dbus_property_at(DBUS_PATH, interface, property)
}

/// Read a D-Bus property using busctl from a specific path
fn read_dbus_property_at(path: &str, interface: &str, property: &str) -> Result<String> {
    let output = Command::new("busctl")
        .args(["get-property", DBUS_DEST, path, interface, property])
        .output()
        .map_err(|e| AsusctlError::CommandFailed(format!("busctl failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such") || stderr.contains("not found") {
            return Err(AsusctlError::ServiceNotRunning);
        }
        return Err(AsusctlError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Parse a boolean property from busctl output (format: "b true" or "b false")
fn parse_dbus_bool(output: &str) -> Result<bool> {
    let value = output
        .strip_prefix("b ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected boolean, got: {}", output)))?;

    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(AsusctlError::ParseError(format!(
            "Invalid boolean value: {}",
            value
        ))),
    }
}

/// Parse a byte property from busctl output (format: "y 255")
fn parse_dbus_byte(output: &str) -> Result<u8> {
    let value = output
        .strip_prefix("y ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected byte, got: {}", output)))?;

    value
        .parse()
        .map_err(|_| AsusctlError::ParseError(format!("Invalid byte value: {}", value)))
}

/// Parse an unsigned int property from busctl output (format: "u 2")
fn parse_dbus_uint(output: &str) -> Result<u32> {
    let value = output
        .strip_prefix("u ")
        .ok_or_else(|| AsusctlError::ParseError(format!("Expected uint, got: {}", output)))?;

    value
        .parse()
        .map_err(|_| AsusctlError::ParseError(format!("Invalid uint value: {}", value)))
}

// ============================================================================
// Platform D-Bus API (profiles, charge limit)
// ============================================================================

/// Get current platform profile via D-Bus (returns 0=Quiet, 1=Balanced, 2=Performance)
pub fn get_platform_profile_dbus() -> Result<PowerProfile> {
    let output = read_dbus_property(PLATFORM_INTERFACE, "PlatformProfile")?;
    let value = parse_dbus_uint(&output)?;

    match value {
        0 => Ok(PowerProfile::Quiet),
        1 => Ok(PowerProfile::Balanced),
        2 => Ok(PowerProfile::Performance),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown profile value: {}",
            value
        ))),
    }
}

/// Get charge control threshold via D-Bus
pub fn get_charge_limit_dbus() -> Result<u8> {
    let output = read_dbus_property(PLATFORM_INTERFACE, "ChargeControlEndThreshold")?;
    parse_dbus_byte(&output)
}

/// Get profile on AC via D-Bus
pub fn get_profile_on_ac_dbus() -> Result<PowerProfile> {
    let output = read_dbus_property(PLATFORM_INTERFACE, "PlatformProfileOnAc")?;
    let value = parse_dbus_uint(&output)?;

    match value {
        0 => Ok(PowerProfile::Quiet),
        1 => Ok(PowerProfile::Balanced),
        2 => Ok(PowerProfile::Performance),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown profile value: {}",
            value
        ))),
    }
}

/// Get profile on battery via D-Bus
pub fn get_profile_on_battery_dbus() -> Result<PowerProfile> {
    let output = read_dbus_property(PLATFORM_INTERFACE, "PlatformProfileOnBattery")?;
    let value = parse_dbus_uint(&output)?;

    match value {
        0 => Ok(PowerProfile::Quiet),
        1 => Ok(PowerProfile::Balanced),
        2 => Ok(PowerProfile::Performance),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown profile value: {}",
            value
        ))),
    }
}

/// Set charge limit (20-100)
pub fn set_charge_limit(limit: u8) -> Result<()> {
    run_asusctl(&["--chg-limit", &limit.to_string()])?;
    Ok(())
}

// ============================================================================
// Aura D-Bus API (keyboard brightness)
// ============================================================================

/// Get current keyboard brightness via D-Bus (0=Off, 1=Low, 2=Med, 3=High)
pub fn get_keyboard_brightness_dbus() -> Result<KeyboardBrightness> {
    let output = read_dbus_property_at(AURA_PATH, AURA_INTERFACE, "Brightness")?;
    let value = parse_dbus_uint(&output)?;

    match value {
        0 => Ok(KeyboardBrightness::Off),
        1 => Ok(KeyboardBrightness::Low),
        2 => Ok(KeyboardBrightness::Med),
        3 => Ok(KeyboardBrightness::High),
        _ => Err(AsusctlError::ParseError(format!(
            "Unknown brightness value: {}",
            value
        ))),
    }
}

// ============================================================================
// Slash D-Bus API (LED bar)
// ============================================================================

/// Get slash enabled state via D-Bus
pub fn get_slash_enabled_dbus() -> Result<bool> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "Enabled")?;
    parse_dbus_bool(&output)
}

/// Get slash brightness via D-Bus (0-255)
pub fn get_slash_brightness_dbus() -> Result<u8> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "Brightness")?;
    parse_dbus_byte(&output)
}

/// Get slash interval via D-Bus
pub fn get_slash_interval_dbus() -> Result<u8> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "Interval")?;
    parse_dbus_byte(&output)
}

/// Get slash mode via D-Bus (returns numeric mode)
pub fn get_slash_mode_dbus() -> Result<u8> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "Mode")?;
    parse_dbus_byte(&output)
}

/// Get slash ShowOnBoot via D-Bus
pub fn get_slash_show_on_boot() -> Result<bool> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "ShowOnBoot")?;
    parse_dbus_bool(&output)
}

/// Get slash ShowOnShutdown via D-Bus
pub fn get_slash_show_on_shutdown() -> Result<bool> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "ShowOnShutdown")?;
    parse_dbus_bool(&output)
}

/// Get slash ShowOnSleep via D-Bus
pub fn get_slash_show_on_sleep() -> Result<bool> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "ShowOnSleep")?;
    parse_dbus_bool(&output)
}

/// Get slash ShowOnBattery via D-Bus
pub fn get_slash_show_on_battery() -> Result<bool> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "ShowOnBattery")?;
    parse_dbus_bool(&output)
}

/// Get slash ShowBatteryWarning via D-Bus
pub fn get_slash_show_battery_warning() -> Result<bool> {
    let output = read_dbus_property_at(SLASH_PATH, SLASH_INTERFACE, "ShowBatteryWarning")?;
    parse_dbus_bool(&output)
}

/// Set slash ShowOnBoot
pub fn set_slash_show_on_boot(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-boot",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

/// Set slash ShowOnShutdown
pub fn set_slash_show_on_shutdown(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-shutdown",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

/// Set slash ShowOnSleep
pub fn set_slash_show_on_sleep(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-sleep",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

/// Set slash ShowOnBattery
pub fn set_slash_show_on_battery(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-on-battery",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

/// Set slash ShowBatteryWarning
pub fn set_slash_show_battery_warning(value: bool) -> Result<()> {
    run_asusctl(&[
        "slash",
        "--show-battery-warning",
        if value { "true" } else { "false" },
    ])?;
    Ok(())
}

// ============================================================================
// Slash Config File Parsing
// ============================================================================

/// Parse slash config from /etc/asusd/slash.ron
fn parse_slash_config() -> Result<SlashState> {
    let content = fs::read_to_string(SLASH_CONFIG_PATH)
        .map_err(|e| AsusctlError::ParseError(format!("Failed to read slash config: {}", e)))?;

    let mut state = SlashState::default();

    // Simple RON parsing - look for key patterns
    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("enabled:") {
            state.enabled = line.contains("true");
        } else if line.starts_with("brightness:") {
            if let Some(val) = extract_number(line) {
                state.brightness = val as u8;
            }
        } else if line.starts_with("display_interval:") {
            if let Some(val) = extract_number(line) {
                state.interval = val as u8;
            }
        } else if line.starts_with("display_mode:") {
            if let Some(mode_str) = extract_string_value(line) {
                state.mode = SlashMode::from_str(&mode_str).unwrap_or_default();
            }
        }
    }

    Ok(state)
}

/// Extract a number from a line like "brightness: 255,"
fn extract_number(line: &str) -> Option<u32> {
    line.split(':')
        .nth(1)?
        .trim()
        .trim_end_matches(',')
        .parse()
        .ok()
}

/// Extract a string value from a line like "display_mode: BitStream,"
fn extract_string_value(line: &str) -> Option<String> {
    Some(
        line.split(':')
            .nth(1)?
            .trim()
            .trim_end_matches(',')
            .to_string(),
    )
}

// ============================================================================
// Slash Public API (uses D-Bus, falls back to config file)
// ============================================================================

/// Get current slash state from D-Bus or config file
pub fn get_slash_state() -> Result<SlashState> {
    // Try D-Bus first
    if let (Ok(enabled), Ok(brightness), Ok(interval)) = (
        get_slash_enabled_dbus(),
        get_slash_brightness_dbus(),
        get_slash_interval_dbus(),
    ) {
        return Ok(SlashState {
            enabled,
            brightness,
            interval,
            mode: SlashMode::default(), // Mode from D-Bus is numeric, harder to map
        });
    }

    // Fall back to config file
    parse_slash_config()
}

/// Get slash enabled state (D-Bus preferred)
pub fn get_slash_enabled() -> Result<bool> {
    get_slash_enabled_dbus().or_else(|_| Ok(parse_slash_config()?.enabled))
}

/// Get slash brightness (D-Bus preferred)
pub fn get_slash_brightness() -> Result<u8> {
    get_slash_brightness_dbus().or_else(|_| Ok(parse_slash_config()?.brightness))
}

/// Get slash interval (D-Bus preferred)
pub fn get_slash_interval() -> Result<u8> {
    get_slash_interval_dbus().or_else(|_| Ok(parse_slash_config()?.interval))
}

/// Get slash mode
pub fn get_slash_mode() -> Result<SlashMode> {
    Ok(parse_slash_config()?.mode)
}

/// Set slash mode
pub fn set_slash_mode(mode: SlashMode) -> Result<()> {
    run_asusctl(&["slash", "--mode", &mode.to_string()])?;
    Ok(())
}

/// Set slash interval (0-5)
pub fn set_slash_interval(interval: u8) -> Result<()> {
    run_asusctl(&["slash", "--interval", &interval.to_string()])?;
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_system_info() {
        let output = r#"Starting version 6.2.0
asusctl v6.2.0
asusctl version: 6.2.0
 Product family: ROG Zephyrus G14
     Board name: GA403UV"#;

        let info = parse_system_info(output).unwrap();
        assert_eq!(info.asusctl_version, "6.2.0");
        assert_eq!(info.product_family, "ROG Zephyrus G14");
        assert_eq!(info.board_name, "GA403UV");
    }

    #[test]
    fn test_parse_keyboard_brightness() {
        let output = "Starting version 6.2.0\nCurrent keyboard led brightness: High";
        let brightness = parse_keyboard_brightness(output).unwrap();
        assert_eq!(brightness, KeyboardBrightness::High);
    }

    #[test]
    fn test_parse_profile_state() {
        let output = r#"Starting version 6.2.0
Active profile is Quiet
Profile on AC is Quiet
Profile on Battery is Quiet"#;

        let state = parse_profile_state(output).unwrap();
        assert_eq!(state.active, PowerProfile::Quiet);
        assert_eq!(state.on_ac, PowerProfile::Quiet);
        assert_eq!(state.on_battery, PowerProfile::Quiet);
    }

    #[test]
    fn test_brightness_from_str() {
        assert_eq!(
            KeyboardBrightness::from_str("High").unwrap(),
            KeyboardBrightness::High
        );
        assert_eq!(
            KeyboardBrightness::from_str("off").unwrap(),
            KeyboardBrightness::Off
        );
    }
}
