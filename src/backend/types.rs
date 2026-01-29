//! Type definitions for the asusctl backend.

use std::str::FromStr;

use super::error::{AsusctlError, Result};

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
                "Unknown brightness level: {s}"
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
                "Unknown power profile: {s}"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuraMode {
    #[default]
    Static,
    Breathe,
    RainbowCycle,
    RainbowWave,
    Highlight,
}

impl AuraMode {
    /// Returns the CLI subcommand name for `asusctl aura <mode>`
    pub fn cli_name(&self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Breathe => "breathe",
            Self::RainbowCycle => "rainbow-cycle",
            Self::RainbowWave => "rainbow-wave",
            Self::Highlight => "highlight",
        }
    }

    /// All available modes
    pub const ALL: &'static [AuraMode] = &[
        Self::Static,
        Self::Breathe,
        Self::RainbowCycle,
        Self::RainbowWave,
        Self::Highlight,
    ];

    /// Short UI label (matches CLI subcommand names)
    pub fn label(&self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Breathe => "Breathe",
            Self::RainbowCycle => "Rainbow Cycle",
            Self::RainbowWave => "Rainbow Wave",
            Self::Highlight => "Highlight",
        }
    }

    /// Whether this mode requires a colour parameter
    pub fn needs_colour(&self) -> bool {
        matches!(self, Self::Static | Self::Breathe | Self::Highlight)
    }

    /// Whether this mode requires a second colour parameter
    pub fn needs_colour2(&self) -> bool {
        matches!(self, Self::Breathe)
    }

    /// Whether this mode requires a speed parameter
    pub fn needs_speed(&self) -> bool {
        matches!(
            self,
            Self::Breathe | Self::RainbowCycle | Self::RainbowWave | Self::Highlight
        )
    }

    /// Whether this mode requires a direction parameter
    pub fn needs_direction(&self) -> bool {
        matches!(self, Self::RainbowWave)
    }

    /// Convert from D-Bus LedMode numeric value to AuraMode
    ///
    /// Based on asusd's AuraEffect enum (rog-aura crate):
    /// Static = 0, Breathe = 1, Strobe = 2, Rainbow = 3, Star = 4, Rain = 5,
    /// Highlight = 6, Laser = 7, Ripple = 8, Pulse = 9, Comet = 10, Flash = 11
    pub fn from_dbus_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Static),
            1 => Some(Self::Breathe),
            3 => Some(Self::RainbowCycle), // Rainbow in asusd
            4 => Some(Self::RainbowWave),  // Star/RainbowWave in some versions
            6 => Some(Self::Highlight),
            _ => None,
        }
    }
}

impl std::fmt::Display for AuraMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static => write!(f, "Static"),
            Self::Breathe => write!(f, "Breathe"),
            Self::RainbowCycle => write!(f, "Rainbow Cycle"),
            Self::RainbowWave => write!(f, "Rainbow Wave"),
            Self::Highlight => write!(f, "Highlight"),
        }
    }
}

impl FromStr for AuraMode {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "static" => Ok(Self::Static),
            "breathe" => Ok(Self::Breathe),
            "rainbow-cycle" | "rainbowcycle" | "rainbow cycle" => Ok(Self::RainbowCycle),
            "rainbow-wave" | "rainbowwave" | "rainbow wave" => Ok(Self::RainbowWave),
            "highlight" => Ok(Self::Highlight),
            _ => Err(AsusctlError::ParseError(format!("Unknown aura mode: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuraSpeed {
    Low,
    #[default]
    Med,
    High,
}

impl std::fmt::Display for AuraSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Med => write!(f, "med"),
            Self::High => write!(f, "high"),
        }
    }
}

impl FromStr for AuraSpeed {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "med" => Ok(Self::Med),
            "high" => Ok(Self::High),
            _ => Err(AsusctlError::ParseError(format!("Unknown aura speed: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuraDirection {
    Up,
    Down,
    Left,
    #[default]
    Right,
}

impl std::fmt::Display for AuraDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "up"),
            Self::Down => write!(f, "down"),
            Self::Left => write!(f, "left"),
            Self::Right => write!(f, "right"),
        }
    }
}

impl FromStr for AuraDirection {
    type Err = AsusctlError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(AsusctlError::ParseError(format!(
                "Unknown aura direction: {s}"
            ))),
        }
    }
}

/// Full aura mode state read from D-Bus LedModeData property
#[derive(Debug, Clone, Default)]
pub struct AuraModeData {
    pub mode: AuraMode,
    #[allow(dead_code)] // Reserved for future multi-zone support
    pub zone: u32,
    pub color1: (u8, u8, u8),
    pub color2: (u8, u8, u8),
    pub speed: AuraSpeed,
    pub direction: AuraDirection,
}

// ============================================================================
// Slash Mode
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
            _ => Err(AsusctlError::ParseError(format!("Unknown slash mode: {s}"))),
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
// Supported Features
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SupportedFeatures {
    pub has_aura: bool,
    pub has_platform: bool,
    pub has_fan_curves: bool,
    pub has_slash: bool,
    pub keyboard_brightness_levels: Vec<KeyboardBrightness>,
    pub aura_modes: Vec<AuraMode>,
    pub aura_zones: Vec<String>,
    pub has_charge_control: bool,
    pub has_throttle_policy: bool,
}

// ============================================================================
// System Info
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub asusctl_version: String,
    pub product_family: String,
    pub board_name: String,
}
