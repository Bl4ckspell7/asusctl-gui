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
    Stars,
    Rain,
    Highlight,
    Laser,
    Ripple,
    Pulse,
    Comet,
    Flash,
}

impl AuraMode {
    /// Returns the CLI subcommand name for `asusctl aura effect <mode>`
    pub fn cli_name(&self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Breathe => "breathe",
            Self::RainbowCycle => "rainbow-cycle",
            Self::RainbowWave => "rainbow-wave",
            Self::Stars => "stars",
            Self::Rain => "rain",
            Self::Highlight => "highlight",
            Self::Laser => "laser",
            Self::Ripple => "ripple",
            Self::Pulse => "pulse",
            Self::Comet => "comet",
            Self::Flash => "flash",
        }
    }

    /// All available modes
    pub const ALL: &'static [AuraMode] = &[
        Self::Static,
        Self::Breathe,
        Self::RainbowCycle,
        Self::RainbowWave,
        Self::Stars,
        Self::Rain,
        Self::Highlight,
        Self::Laser,
        Self::Ripple,
        Self::Pulse,
        Self::Comet,
        Self::Flash,
    ];

    /// Short UI label (matches CLI subcommand names)
    pub fn label(&self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Breathe => "Breathe",
            Self::RainbowCycle => "Rainbow Cycle",
            Self::RainbowWave => "Rainbow Wave",
            Self::Stars => "Stars",
            Self::Rain => "Rain",
            Self::Highlight => "Highlight",
            Self::Laser => "Laser",
            Self::Ripple => "Ripple",
            Self::Pulse => "Pulse",
            Self::Comet => "Comet",
            Self::Flash => "Flash",
        }
    }

    /// Whether this mode requires a colour parameter
    pub fn needs_colour(&self) -> bool {
        matches!(
            self,
            Self::Static
                | Self::Breathe
                | Self::Stars
                | Self::Highlight
                | Self::Laser
                | Self::Ripple
                | Self::Pulse
                | Self::Comet
                | Self::Flash
        )
    }

    /// Whether this mode requires a second colour parameter
    pub fn needs_colour2(&self) -> bool {
        matches!(self, Self::Breathe | Self::Stars)
    }

    /// Whether this mode requires a speed parameter
    pub fn needs_speed(&self) -> bool {
        matches!(
            self,
            Self::Breathe
                | Self::RainbowCycle
                | Self::RainbowWave
                | Self::Stars
                | Self::Rain
                | Self::Highlight
                | Self::Laser
                | Self::Ripple
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
            3 => Some(Self::RainbowCycle),
            4 => Some(Self::Stars),
            5 => Some(Self::Rain),
            6 => Some(Self::Highlight),
            7 => Some(Self::Laser),
            8 => Some(Self::Ripple),
            9 => Some(Self::Pulse),
            10 => Some(Self::Comet),
            11 => Some(Self::Flash),
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
            Self::Stars => write!(f, "Stars"),
            Self::Rain => write!(f, "Rain"),
            Self::Highlight => write!(f, "Highlight"),
            Self::Laser => write!(f, "Laser"),
            Self::Ripple => write!(f, "Ripple"),
            Self::Pulse => write!(f, "Pulse"),
            Self::Comet => write!(f, "Comet"),
            Self::Flash => write!(f, "Flash"),
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
            "stars" => Ok(Self::Stars),
            "rain" => Ok(Self::Rain),
            "highlight" => Ok(Self::Highlight),
            "laser" => Ok(Self::Laser),
            "ripple" => Ok(Self::Ripple),
            "pulse" => Ok(Self::Pulse),
            "comet" => Ok(Self::Comet),
            "flash" => Ok(Self::Flash),
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
    Static,
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
            Self::Static => write!(f, "Static"),
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
            "Static" => Ok(Self::Static),
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
    pub asusctl_installed: bool,
    pub asusd_running: bool,
    pub has_aura: bool,
    pub has_platform: bool,
    pub has_fan_curves: bool,
    pub has_slash: bool,
    pub keyboard_brightness_levels: Vec<KeyboardBrightness>,
    pub aura_modes: Vec<AuraMode>,
    pub aura_zones: Vec<String>,
    pub has_charge_control: bool,
    pub has_throttle_policy: bool,
    pub has_armoury: bool,
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // KeyboardBrightness Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_keyboard_brightness_from_str_valid() {
        assert_eq!(
            KeyboardBrightness::from_str("off").unwrap(),
            KeyboardBrightness::Off
        );
        assert_eq!(
            KeyboardBrightness::from_str("OFF").unwrap(),
            KeyboardBrightness::Off
        );
        assert_eq!(
            KeyboardBrightness::from_str("low").unwrap(),
            KeyboardBrightness::Low
        );
        assert_eq!(
            KeyboardBrightness::from_str("Low").unwrap(),
            KeyboardBrightness::Low
        );
        assert_eq!(
            KeyboardBrightness::from_str("med").unwrap(),
            KeyboardBrightness::Med
        );
        assert_eq!(
            KeyboardBrightness::from_str("MED").unwrap(),
            KeyboardBrightness::Med
        );
        assert_eq!(
            KeyboardBrightness::from_str("high").unwrap(),
            KeyboardBrightness::High
        );
        assert_eq!(
            KeyboardBrightness::from_str("HIGH").unwrap(),
            KeyboardBrightness::High
        );
    }

    #[test]
    fn test_keyboard_brightness_from_str_invalid() {
        assert!(KeyboardBrightness::from_str("invalid").is_err());
        assert!(KeyboardBrightness::from_str("").is_err());
        assert!(KeyboardBrightness::from_str("medium").is_err());
    }

    #[test]
    fn test_keyboard_brightness_display() {
        assert_eq!(KeyboardBrightness::Off.to_string(), "off");
        assert_eq!(KeyboardBrightness::Low.to_string(), "low");
        assert_eq!(KeyboardBrightness::Med.to_string(), "med");
        assert_eq!(KeyboardBrightness::High.to_string(), "high");
    }

    #[test]
    fn test_keyboard_brightness_default() {
        assert_eq!(KeyboardBrightness::default(), KeyboardBrightness::High);
    }

    // ------------------------------------------------------------------------
    // PowerProfile Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_power_profile_from_str_valid() {
        assert_eq!(
            PowerProfile::from_str("quiet").unwrap(),
            PowerProfile::Quiet
        );
        assert_eq!(
            PowerProfile::from_str("QUIET").unwrap(),
            PowerProfile::Quiet
        );
        assert_eq!(
            PowerProfile::from_str("balanced").unwrap(),
            PowerProfile::Balanced
        );
        assert_eq!(
            PowerProfile::from_str("Balanced").unwrap(),
            PowerProfile::Balanced
        );
        assert_eq!(
            PowerProfile::from_str("performance").unwrap(),
            PowerProfile::Performance
        );
        assert_eq!(
            PowerProfile::from_str("PERFORMANCE").unwrap(),
            PowerProfile::Performance
        );
    }

    #[test]
    fn test_power_profile_from_str_invalid() {
        assert!(PowerProfile::from_str("turbo").is_err());
        assert!(PowerProfile::from_str("").is_err());
        assert!(PowerProfile::from_str("power-saver").is_err());
    }

    #[test]
    fn test_power_profile_display() {
        assert_eq!(PowerProfile::Quiet.to_string(), "Quiet");
        assert_eq!(PowerProfile::Balanced.to_string(), "Balanced");
        assert_eq!(PowerProfile::Performance.to_string(), "Performance");
    }

    #[test]
    fn test_power_profile_default() {
        assert_eq!(PowerProfile::default(), PowerProfile::Balanced);
    }

    // ------------------------------------------------------------------------
    // AuraMode Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_aura_mode_from_str_valid() {
        assert_eq!(AuraMode::from_str("static").unwrap(), AuraMode::Static);
        assert_eq!(AuraMode::from_str("Static").unwrap(), AuraMode::Static);
        assert_eq!(AuraMode::from_str("breathe").unwrap(), AuraMode::Breathe);
        assert_eq!(AuraMode::from_str("stars").unwrap(), AuraMode::Stars);
        assert_eq!(AuraMode::from_str("rain").unwrap(), AuraMode::Rain);
        assert_eq!(
            AuraMode::from_str("highlight").unwrap(),
            AuraMode::Highlight
        );
        assert_eq!(AuraMode::from_str("laser").unwrap(), AuraMode::Laser);
        assert_eq!(AuraMode::from_str("ripple").unwrap(), AuraMode::Ripple);
        assert_eq!(AuraMode::from_str("pulse").unwrap(), AuraMode::Pulse);
        assert_eq!(AuraMode::from_str("comet").unwrap(), AuraMode::Comet);
        assert_eq!(AuraMode::from_str("flash").unwrap(), AuraMode::Flash);
    }

    #[test]
    fn test_aura_mode_from_str_rainbow_variants() {
        // Test all accepted formats for rainbow modes
        assert_eq!(
            AuraMode::from_str("rainbow-cycle").unwrap(),
            AuraMode::RainbowCycle
        );
        assert_eq!(
            AuraMode::from_str("rainbowcycle").unwrap(),
            AuraMode::RainbowCycle
        );
        assert_eq!(
            AuraMode::from_str("rainbow cycle").unwrap(),
            AuraMode::RainbowCycle
        );
        assert_eq!(
            AuraMode::from_str("rainbow-wave").unwrap(),
            AuraMode::RainbowWave
        );
        assert_eq!(
            AuraMode::from_str("rainbowwave").unwrap(),
            AuraMode::RainbowWave
        );
        assert_eq!(
            AuraMode::from_str("rainbow wave").unwrap(),
            AuraMode::RainbowWave
        );
    }

    #[test]
    fn test_aura_mode_from_str_invalid() {
        assert!(AuraMode::from_str("invalid").is_err());
        assert!(AuraMode::from_str("").is_err());
        assert!(AuraMode::from_str("strobe").is_err());
    }

    #[test]
    fn test_aura_mode_cli_name() {
        assert_eq!(AuraMode::Static.cli_name(), "static");
        assert_eq!(AuraMode::Breathe.cli_name(), "breathe");
        assert_eq!(AuraMode::RainbowCycle.cli_name(), "rainbow-cycle");
        assert_eq!(AuraMode::RainbowWave.cli_name(), "rainbow-wave");
        assert_eq!(AuraMode::Stars.cli_name(), "stars");
        assert_eq!(AuraMode::Rain.cli_name(), "rain");
        assert_eq!(AuraMode::Highlight.cli_name(), "highlight");
        assert_eq!(AuraMode::Laser.cli_name(), "laser");
        assert_eq!(AuraMode::Ripple.cli_name(), "ripple");
        assert_eq!(AuraMode::Pulse.cli_name(), "pulse");
        assert_eq!(AuraMode::Comet.cli_name(), "comet");
        assert_eq!(AuraMode::Flash.cli_name(), "flash");
    }

    #[test]
    fn test_aura_mode_label() {
        assert_eq!(AuraMode::Static.label(), "Static");
        assert_eq!(AuraMode::RainbowCycle.label(), "Rainbow Cycle");
        assert_eq!(AuraMode::RainbowWave.label(), "Rainbow Wave");
    }

    #[test]
    fn test_aura_mode_needs_colour() {
        // Modes that need colour
        assert!(AuraMode::Static.needs_colour());
        assert!(AuraMode::Breathe.needs_colour());
        assert!(AuraMode::Stars.needs_colour());
        assert!(AuraMode::Highlight.needs_colour());
        assert!(AuraMode::Laser.needs_colour());
        assert!(AuraMode::Ripple.needs_colour());
        assert!(AuraMode::Pulse.needs_colour());
        assert!(AuraMode::Comet.needs_colour());
        assert!(AuraMode::Flash.needs_colour());

        // Modes that don't need colour
        assert!(!AuraMode::RainbowCycle.needs_colour());
        assert!(!AuraMode::RainbowWave.needs_colour());
        assert!(!AuraMode::Rain.needs_colour());
    }

    #[test]
    fn test_aura_mode_needs_colour2() {
        // Only Breathe and Stars need a second colour
        assert!(AuraMode::Breathe.needs_colour2());
        assert!(AuraMode::Stars.needs_colour2());

        // Others don't
        assert!(!AuraMode::Static.needs_colour2());
        assert!(!AuraMode::RainbowCycle.needs_colour2());
        assert!(!AuraMode::Laser.needs_colour2());
    }

    #[test]
    fn test_aura_mode_needs_speed() {
        // Modes that need speed
        assert!(AuraMode::Breathe.needs_speed());
        assert!(AuraMode::RainbowCycle.needs_speed());
        assert!(AuraMode::RainbowWave.needs_speed());
        assert!(AuraMode::Stars.needs_speed());
        assert!(AuraMode::Rain.needs_speed());
        assert!(AuraMode::Highlight.needs_speed());
        assert!(AuraMode::Laser.needs_speed());
        assert!(AuraMode::Ripple.needs_speed());

        // Modes that don't need speed
        assert!(!AuraMode::Static.needs_speed());
        assert!(!AuraMode::Pulse.needs_speed());
        assert!(!AuraMode::Comet.needs_speed());
        assert!(!AuraMode::Flash.needs_speed());
    }

    #[test]
    fn test_aura_mode_needs_direction() {
        // Only RainbowWave needs direction
        assert!(AuraMode::RainbowWave.needs_direction());

        // Others don't
        assert!(!AuraMode::Static.needs_direction());
        assert!(!AuraMode::RainbowCycle.needs_direction());
        assert!(!AuraMode::Breathe.needs_direction());
    }

    #[test]
    fn test_aura_mode_from_dbus_value() {
        assert_eq!(AuraMode::from_dbus_value(0), Some(AuraMode::Static));
        assert_eq!(AuraMode::from_dbus_value(1), Some(AuraMode::Breathe));
        assert_eq!(AuraMode::from_dbus_value(3), Some(AuraMode::RainbowCycle));
        assert_eq!(AuraMode::from_dbus_value(4), Some(AuraMode::Stars));
        assert_eq!(AuraMode::from_dbus_value(5), Some(AuraMode::Rain));
        assert_eq!(AuraMode::from_dbus_value(6), Some(AuraMode::Highlight));
        assert_eq!(AuraMode::from_dbus_value(7), Some(AuraMode::Laser));
        assert_eq!(AuraMode::from_dbus_value(8), Some(AuraMode::Ripple));
        assert_eq!(AuraMode::from_dbus_value(9), Some(AuraMode::Pulse));
        assert_eq!(AuraMode::from_dbus_value(10), Some(AuraMode::Comet));
        assert_eq!(AuraMode::from_dbus_value(11), Some(AuraMode::Flash));

        // Invalid values
        assert_eq!(AuraMode::from_dbus_value(2), None); // Strobe is not mapped
        assert_eq!(AuraMode::from_dbus_value(12), None);
        assert_eq!(AuraMode::from_dbus_value(100), None);
    }

    #[test]
    fn test_aura_mode_display() {
        assert_eq!(AuraMode::Static.to_string(), "Static");
        assert_eq!(AuraMode::RainbowCycle.to_string(), "Rainbow Cycle");
        assert_eq!(AuraMode::RainbowWave.to_string(), "Rainbow Wave");
    }

    #[test]
    fn test_aura_mode_default() {
        assert_eq!(AuraMode::default(), AuraMode::Static);
    }

    #[test]
    fn test_aura_mode_all_constant() {
        assert_eq!(AuraMode::ALL.len(), 12);
        assert!(AuraMode::ALL.contains(&AuraMode::Static));
        assert!(AuraMode::ALL.contains(&AuraMode::Flash));
    }

    // ------------------------------------------------------------------------
    // AuraSpeed Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_aura_speed_from_str_valid() {
        assert_eq!(AuraSpeed::from_str("low").unwrap(), AuraSpeed::Low);
        assert_eq!(AuraSpeed::from_str("LOW").unwrap(), AuraSpeed::Low);
        assert_eq!(AuraSpeed::from_str("med").unwrap(), AuraSpeed::Med);
        assert_eq!(AuraSpeed::from_str("Med").unwrap(), AuraSpeed::Med);
        assert_eq!(AuraSpeed::from_str("high").unwrap(), AuraSpeed::High);
        assert_eq!(AuraSpeed::from_str("HIGH").unwrap(), AuraSpeed::High);
    }

    #[test]
    fn test_aura_speed_from_str_invalid() {
        assert!(AuraSpeed::from_str("fast").is_err());
        assert!(AuraSpeed::from_str("").is_err());
        assert!(AuraSpeed::from_str("medium").is_err());
    }

    #[test]
    fn test_aura_speed_display() {
        assert_eq!(AuraSpeed::Low.to_string(), "low");
        assert_eq!(AuraSpeed::Med.to_string(), "med");
        assert_eq!(AuraSpeed::High.to_string(), "high");
    }

    #[test]
    fn test_aura_speed_default() {
        assert_eq!(AuraSpeed::default(), AuraSpeed::Med);
    }

    // ------------------------------------------------------------------------
    // AuraDirection Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_aura_direction_from_str_valid() {
        assert_eq!(AuraDirection::from_str("up").unwrap(), AuraDirection::Up);
        assert_eq!(AuraDirection::from_str("UP").unwrap(), AuraDirection::Up);
        assert_eq!(
            AuraDirection::from_str("down").unwrap(),
            AuraDirection::Down
        );
        assert_eq!(
            AuraDirection::from_str("Down").unwrap(),
            AuraDirection::Down
        );
        assert_eq!(
            AuraDirection::from_str("left").unwrap(),
            AuraDirection::Left
        );
        assert_eq!(
            AuraDirection::from_str("LEFT").unwrap(),
            AuraDirection::Left
        );
        assert_eq!(
            AuraDirection::from_str("right").unwrap(),
            AuraDirection::Right
        );
        assert_eq!(
            AuraDirection::from_str("Right").unwrap(),
            AuraDirection::Right
        );
    }

    #[test]
    fn test_aura_direction_from_str_invalid() {
        assert!(AuraDirection::from_str("north").is_err());
        assert!(AuraDirection::from_str("").is_err());
    }

    #[test]
    fn test_aura_direction_display() {
        assert_eq!(AuraDirection::Up.to_string(), "up");
        assert_eq!(AuraDirection::Down.to_string(), "down");
        assert_eq!(AuraDirection::Left.to_string(), "left");
        assert_eq!(AuraDirection::Right.to_string(), "right");
    }

    #[test]
    fn test_aura_direction_default() {
        assert_eq!(AuraDirection::default(), AuraDirection::Right);
    }

    // ------------------------------------------------------------------------
    // SlashMode Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_slash_mode_from_str_valid() {
        assert_eq!(SlashMode::from_str("Static").unwrap(), SlashMode::Static);
        assert_eq!(SlashMode::from_str("Bounce").unwrap(), SlashMode::Bounce);
        assert_eq!(SlashMode::from_str("Slash").unwrap(), SlashMode::Slash);
        assert_eq!(SlashMode::from_str("Loading").unwrap(), SlashMode::Loading);
        assert_eq!(
            SlashMode::from_str("BitStream").unwrap(),
            SlashMode::BitStream
        );
        assert_eq!(
            SlashMode::from_str("Transmission").unwrap(),
            SlashMode::Transmission
        );
        assert_eq!(SlashMode::from_str("Flow").unwrap(), SlashMode::Flow);
        assert_eq!(SlashMode::from_str("Flux").unwrap(), SlashMode::Flux);
        assert_eq!(SlashMode::from_str("Phantom").unwrap(), SlashMode::Phantom);
        assert_eq!(
            SlashMode::from_str("Spectrum").unwrap(),
            SlashMode::Spectrum
        );
        assert_eq!(SlashMode::from_str("Hazard").unwrap(), SlashMode::Hazard);
        assert_eq!(
            SlashMode::from_str("Interfacing").unwrap(),
            SlashMode::Interfacing
        );
        assert_eq!(SlashMode::from_str("Ramp").unwrap(), SlashMode::Ramp);
        assert_eq!(
            SlashMode::from_str("GameOver").unwrap(),
            SlashMode::GameOver
        );
        assert_eq!(SlashMode::from_str("Start").unwrap(), SlashMode::Start);
        assert_eq!(SlashMode::from_str("Buzzer").unwrap(), SlashMode::Buzzer);
    }

    #[test]
    fn test_slash_mode_from_str_invalid() {
        assert!(SlashMode::from_str("invalid").is_err());
        assert!(SlashMode::from_str("").is_err());
        // Note: SlashMode uses case-sensitive matching unlike other enums
        assert!(SlashMode::from_str("static").is_err());
        assert!(SlashMode::from_str("STATIC").is_err());
    }

    #[test]
    fn test_slash_mode_display() {
        assert_eq!(SlashMode::Static.to_string(), "Static");
        assert_eq!(SlashMode::BitStream.to_string(), "BitStream");
        assert_eq!(SlashMode::GameOver.to_string(), "GameOver");
    }

    #[test]
    fn test_slash_mode_default() {
        assert_eq!(SlashMode::default(), SlashMode::Flow);
    }

    // ------------------------------------------------------------------------
    // SlashState Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_slash_state_default() {
        let state = SlashState::default();
        assert!(!state.enabled);
        assert_eq!(state.brightness, 0);
        assert_eq!(state.interval, 0);
        assert_eq!(state.mode, SlashMode::Flow);
    }

    // ------------------------------------------------------------------------
    // ProfileState Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_profile_state_default() {
        let state = ProfileState::default();
        assert_eq!(state.active, PowerProfile::Balanced);
        assert_eq!(state.on_ac, PowerProfile::Balanced);
        assert_eq!(state.on_battery, PowerProfile::Balanced);
    }

    // ------------------------------------------------------------------------
    // AuraModeData Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_aura_mode_data_default() {
        let data = AuraModeData::default();
        assert_eq!(data.mode, AuraMode::Static);
        assert_eq!(data.zone, 0);
        assert_eq!(data.color1, (0, 0, 0));
        assert_eq!(data.color2, (0, 0, 0));
        assert_eq!(data.speed, AuraSpeed::Med);
        assert_eq!(data.direction, AuraDirection::Right);
    }

    // ------------------------------------------------------------------------
    // SupportedFeatures Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_supported_features_default() {
        let features = SupportedFeatures::default();
        assert!(!features.asusctl_installed);
        assert!(!features.asusd_running);
        assert!(!features.has_aura);
        assert!(!features.has_platform);
        assert!(!features.has_fan_curves);
        assert!(!features.has_slash);
        assert!(features.keyboard_brightness_levels.is_empty());
        assert!(features.aura_modes.is_empty());
        assert!(features.aura_zones.is_empty());
        assert!(!features.has_charge_control);
        assert!(!features.has_throttle_policy);
        assert!(!features.has_armoury);
    }

    // ------------------------------------------------------------------------
    // SystemInfo Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_system_info_default() {
        let info = SystemInfo::default();
        assert!(info.asusctl_version.is_empty());
        assert!(info.product_family.is_empty());
        assert!(info.board_name.is_empty());
    }
}
