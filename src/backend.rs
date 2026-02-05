//! Backend module for wrapping asusctl CLI commands.
//!
//! This module provides structured Rust types and functions for interacting
//! with asusctl, handling errors gracefully when asusctl is not installed
//! or the asusd service is not running.
//!
//! State reading strategy:
//! - Platform (profiles, charge limit): D-Bus via xyz.ljones.Platform
//! - Slash: Config file at /etc/asusd/slash.ron (D-Bus fallback)
//! - Aura/Keyboard brightness: D-Bus via xyz.ljones.Aura
//!
//! # Module Structure
//!
//! - [`error`] - Error types and Result alias
//! - [`types`] - Data structures (enums, structs)
//! - [`dbus`] - D-Bus communication helpers (internal)
//! - [`system`] - System info and feature detection
//! - [`aura`] - Keyboard brightness and lighting control
//! - [`power`] - Power profiles and charge control
//! - [`slash`] - LED bar control

mod aura;
mod dbus;
mod error;
mod power;
mod slash;
mod system;
mod types;

// Re-export types actually used by UI
pub use types::{
    AuraDirection, AuraMode, AuraSpeed, KeyboardBrightness, PowerProfile, SlashMode,
    SupportedFeatures,
};

// Re-export system functions
pub use system::{detect_features, get_system_info};

// Re-export aura functions
pub use aura::{
    get_aura_mode_data_dbus, get_aura_mode_help, get_keyboard_brightness_dbus, is_rainbow_running,
    set_aura_mode, set_keyboard_brightness, start_rainbow, stop_rainbow,
};

// Re-export power functions
pub use power::{
    get_charge_limit_dbus, get_profile_state, set_charge_limit, set_profile, set_profile_ac,
    set_profile_battery,
};

// Re-export slash functions
pub use slash::{
    disable_slash, enable_slash, get_slash_brightness, get_slash_enabled, get_slash_interval,
    get_slash_mode, get_slash_show_battery_warning, get_slash_show_on_battery,
    get_slash_show_on_boot, get_slash_show_on_shutdown, get_slash_show_on_sleep,
    set_slash_brightness, set_slash_interval, set_slash_mode, set_slash_show_battery_warning,
    set_slash_show_on_battery, set_slash_show_on_boot, set_slash_show_on_shutdown,
    set_slash_show_on_sleep,
};
