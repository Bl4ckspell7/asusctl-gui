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
//! - [`dmi`] - DMI/SMBIOS information from sysfs
//! - [`system`] - System info and feature detection
//! - [`aura`] - Keyboard brightness and lighting control
//! - [`power`] - Power profiles and charge control
//! - [`gpu`] - GPU mode switching (dGPU / MUX)
//! - [`slash`] - LED bar control
//! - [`shortcut`] - GNOME custom shortcut that launches the app

mod aura;
mod dbus;
mod dmi;
mod error;
mod gpu;
mod power;
mod shortcut;
mod slash;
mod system;
mod types;

pub(crate) use error::AsusctlError;

// Re-export types actually used by UI
pub use types::{
    AuraDirection, AuraMode, AuraPowerState, AuraPowerZone, AuraSpeed, GpuMode, KeyboardBrightness,
    PowerProfile, SlashMode, SupportedFeatures,
};

// Re-export system functions
pub use system::{detect_features, get_distro, get_kernel_version, get_system_info};

// Re-export DMI functions
pub use dmi::{format_bios, get_dmi_info, read_serial_number};

// Re-export aura functions
pub use aura::{
    get_aura_mode_data_dbus, get_aura_mode_help, get_aura_power_states,
    get_keyboard_brightness_dbus, set_aura_mode, set_aura_power_state, set_keyboard_brightness,
};

// Re-export power functions
pub use power::{
    get_charge_limit_dbus, get_profile_state, set_charge_limit, set_profile, set_profile_ac,
    set_profile_battery,
};

// Re-export GPU functions
pub use gpu::{
    GpuState, available_gpu_modes, get_gpu_state, is_shutdown_service_active, reboot_now,
    request_session_reboot, set_gpu_mode,
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

// Re-export shortcut functions
pub use shortcut::{add_launch_shortcut, get_launch_shortcut, is_gnome, remove_launch_shortcut};
