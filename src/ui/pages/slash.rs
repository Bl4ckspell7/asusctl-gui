use adw::prelude::*;
use gtk4::glib;
use gtk4::glib::prelude::ObjectExt;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, AsusctlError, SlashMode};
use crate::ui::{Refreshable, show_backend_error};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SlashPage {
        pub available: RefCell<bool>,
        pub enable_switch: RefCell<Option<adw::SwitchRow>>,
        pub brightness_scale: RefCell<Option<gtk4::Scale>>,
        pub mode_combo: RefCell<Option<adw::ComboRow>>,
        pub interval_combo: RefCell<Option<adw::ComboRow>>,
        pub show_on_boot: RefCell<Option<adw::SwitchRow>>,
        pub show_on_shutdown: RefCell<Option<adw::SwitchRow>>,
        pub show_on_sleep: RefCell<Option<adw::SwitchRow>>,
        pub show_on_battery: RefCell<Option<adw::SwitchRow>>,
        pub show_battery_warning: RefCell<Option<adw::SwitchRow>>,
        /// Flag to prevent signal handlers from firing during refresh
        pub refreshing: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SlashPage {
        const NAME: &'static str = "SlashPage";
        type Type = super::SlashPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for SlashPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().refresh_data();
        }
    }

    impl WidgetImpl for SlashPage {}
    impl BoxImpl for SlashPage {}
}

glib::wrapper! {
    pub struct SlashPage(ObjectSubclass<imp::SlashPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

// Mode names in order (index matches SlashMode enum variant order)
const SLASH_MODES: &[(&str, &str)] = &[
    ("Static", "Static light effect"),
    ("Bounce", "Bouncing light effect"),
    ("Slash", "Slashing light animation"),
    ("Loading", "Progress bar style animation"),
    ("BitStream", "Digital data stream effect"),
    ("Transmission", "Data transmission visualization"),
    ("Flow", "Flowing light effect"),
    ("Flux", "Pulsing light pattern"),
    ("Phantom", "Ghostly fading effect"),
    ("Spectrum", "Color spectrum animation"),
    ("Hazard", "Warning/hazard style flashing"),
    ("Interfacing", "Interface connection visualization"),
    ("Ramp", "Ramping up/down brightness"),
    ("GameOver", "Game over animation"),
    ("Start", "Startup animation"),
    ("Buzzer", "Alert/notification style animation"),
];

impl SlashPage {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk4::Orientation::Vertical)
            .property("spacing", 24)
            .property("margin-top", 24)
            .property("margin-bottom", 24)
            .property("margin-start", 24)
            .property("margin-end", 24)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();
        let features = backend::detect_features();

        if !features.has_slash {
            *imp.available.borrow_mut() = false;
            let status = adw::StatusPage::builder()
                .icon_name("display-brightness-symbolic")
                .title("Slash Lighting Unavailable")
                .description(if !features.asusctl_installed {
                    "asusctl is not installed. Install asusctl to control the slash LED bar."
                } else {
                    "Slash LED bar is not supported on this device, or the asusd service is not running."
                })
                .vexpand(true)
                .build();
            self.append(&status);
            return;
        }

        *imp.available.borrow_mut() = true;

        // Page title
        let title = gtk4::Label::builder()
            .label("Slash Lighting")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&title);

        // Description
        let description = gtk4::Label::builder()
            .label("Control the LED bar on the back of the laptop display")
            .css_classes(["dim-label"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&description);

        // Power group
        let power_group = adw::PreferencesGroup::builder().title("Power").build();

        let enable_row = adw::SwitchRow::builder()
            .title("Enable Slash Lighting")
            .subtitle("Turn the LED bar on or off")
            .build();

        // Connect the switch to enable/disable slash
        {
            let this = self.clone();
            enable_row.connect_active_notify(move |switch| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let enabled = switch.is_active();
                let result = if enabled {
                    backend::enable_slash()
                } else {
                    backend::disable_slash()
                };

                // Keep slider sensitivity consistent with the optimistic switch state.
                if let Some(scale) = this.imp().brightness_scale.borrow().as_ref() {
                    scale.set_sensitive(enabled);
                }

                if let Err(e) = result {
                    log::error!("Failed to toggle slash: {e}");
                    show_backend_error(&this, "Couldn’t change Slash lighting", &e);
                    this.reconcile_enabled_after_write_failure();
                }
            });
        }

        imp.enable_switch.replace(Some(enable_row.clone()));
        power_group.add(&enable_row);
        self.append(&power_group);

        // Brightness group
        let brightness_group = adw::PreferencesGroup::builder().title("Brightness").build();

        let brightness_row = adw::ActionRow::builder()
            .title("Brightness Level")
            .subtitle("0-255")
            .build();

        let brightness_scale = gtk4::Scale::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .adjustment(&gtk4::Adjustment::new(128.0, 0.0, 255.0, 1.0, 10.0, 0.0))
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .draw_value(true)
            .build();

        // Connect brightness scale to set slash brightness
        {
            let this = self.clone();
            brightness_scale.connect_value_changed(move |scale| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let value = scale.value() as u8;
                if let Err(e) = backend::set_slash_brightness(value) {
                    log::error!("Failed to set slash brightness: {e}");
                    show_backend_error(&this, "Couldn’t change Slash brightness", &e);
                    this.reconcile_brightness_after_write_failure();
                }
            });
        }

        imp.brightness_scale.replace(Some(brightness_scale.clone()));
        brightness_row.add_suffix(&brightness_scale);
        brightness_group.add(&brightness_row);

        self.append(&brightness_group);

        // Mode group
        let mode_group = adw::PreferencesGroup::builder().title("Animation").build();

        // Create mode names list for combo
        let mode_names: Vec<&str> = SLASH_MODES.iter().map(|(name, _)| *name).collect();
        let mode_combo = adw::ComboRow::builder()
            .title("Mode")
            .subtitle("Animation style")
            .model(&gtk4::StringList::new(&mode_names))
            .build();

        // Connect mode combo to set slash mode
        {
            let this = self.clone();
            mode_combo.connect_selected_notify(move |combo| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let mode = match combo.selected() {
                    0 => SlashMode::Static,
                    1 => SlashMode::Bounce,
                    2 => SlashMode::Slash,
                    3 => SlashMode::Loading,
                    4 => SlashMode::BitStream,
                    5 => SlashMode::Transmission,
                    6 => SlashMode::Flow,
                    7 => SlashMode::Flux,
                    8 => SlashMode::Phantom,
                    9 => SlashMode::Spectrum,
                    10 => SlashMode::Hazard,
                    11 => SlashMode::Interfacing,
                    12 => SlashMode::Ramp,
                    13 => SlashMode::GameOver,
                    14 => SlashMode::Start,
                    15 => SlashMode::Buzzer,
                    _ => return,
                };

                if let Err(e) = backend::set_slash_mode(mode) {
                    log::error!("Failed to set slash mode: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash animation", &e);
                    this.reconcile_mode_after_write_failure();
                }
            });
        }

        imp.mode_combo.replace(Some(mode_combo.clone()));
        mode_group.add(&mode_combo);

        // Interval/speed combo
        let interval_combo = adw::ComboRow::builder()
            .title("Speed")
            .subtitle("Animation interval (0 = fastest, 5 = slowest)")
            .model(&gtk4::StringList::new(&["0", "1", "2", "3", "4", "5"]))
            .selected(0)
            .build();

        // Connect interval combo to set slash interval
        {
            let this = self.clone();
            interval_combo.connect_selected_notify(move |combo| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let interval = combo.selected() as u8;
                if let Err(e) = backend::set_slash_interval(interval) {
                    log::error!("Failed to set slash interval: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash animation speed", &e);
                    this.reconcile_interval_after_write_failure();
                }
            });
        }

        imp.interval_combo.replace(Some(interval_combo.clone()));
        mode_group.add(&interval_combo);
        self.append(&mode_group);

        // Show On Events group
        let events_group = adw::PreferencesGroup::builder()
            .title("Show Animation On")
            .description("When to display slash animations")
            .build();

        // Show on boot
        let show_on_boot = adw::SwitchRow::builder()
            .title("Boot")
            .subtitle("Show animation when laptop boots")
            .build();
        {
            let this = self.clone();
            show_on_boot.connect_active_notify(move |switch| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if let Err(e) = backend::set_slash_show_on_boot(switch.is_active()) {
                    log::error!("Failed to set show on boot: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash event settings", &e);
                    this.reconcile_show_on_boot_after_write_failure();
                }
            });
        }
        imp.show_on_boot.replace(Some(show_on_boot.clone()));
        events_group.add(&show_on_boot);

        // Show on shutdown
        let show_on_shutdown = adw::SwitchRow::builder()
            .title("Shutdown")
            .subtitle("Show animation when laptop shuts down")
            .build();
        {
            let this = self.clone();
            show_on_shutdown.connect_active_notify(move |switch| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if let Err(e) = backend::set_slash_show_on_shutdown(switch.is_active()) {
                    log::error!("Failed to set show on shutdown: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash event settings", &e);
                    this.reconcile_show_on_shutdown_after_write_failure();
                }
            });
        }
        imp.show_on_shutdown.replace(Some(show_on_shutdown.clone()));
        events_group.add(&show_on_shutdown);

        // Show on sleep
        let show_on_sleep = adw::SwitchRow::builder()
            .title("Sleep")
            .subtitle("Show animation when laptop sleeps")
            .build();
        {
            let this = self.clone();
            show_on_sleep.connect_active_notify(move |switch| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if let Err(e) = backend::set_slash_show_on_sleep(switch.is_active()) {
                    log::error!("Failed to set show on sleep: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash event settings", &e);
                    this.reconcile_show_on_sleep_after_write_failure();
                }
            });
        }
        imp.show_on_sleep.replace(Some(show_on_sleep.clone()));
        events_group.add(&show_on_sleep);

        // Show on battery
        let show_on_battery = adw::SwitchRow::builder()
            .title("Battery")
            .subtitle("Show animation when on battery power")
            .build();
        {
            let this = self.clone();
            show_on_battery.connect_active_notify(move |switch| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if let Err(e) = backend::set_slash_show_on_battery(switch.is_active()) {
                    log::error!("Failed to set show on battery: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash event settings", &e);
                    this.reconcile_show_on_battery_after_write_failure();
                }
            });
        }
        imp.show_on_battery.replace(Some(show_on_battery.clone()));
        events_group.add(&show_on_battery);

        // Show battery warning
        let show_battery_warning = adw::SwitchRow::builder()
            .title("Low Battery Warning")
            .subtitle("Show animation when battery is low")
            .build();
        {
            let this = self.clone();
            show_battery_warning.connect_active_notify(move |switch| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if let Err(e) = backend::set_slash_show_battery_warning(switch.is_active()) {
                    log::error!("Failed to set show battery warning: {e}");
                    show_backend_error(&this, "Couldn’t change the Slash event settings", &e);
                    this.reconcile_battery_warning_after_write_failure();
                }
            });
        }
        imp.show_battery_warning
            .replace(Some(show_battery_warning.clone()));
        events_group.add(&show_battery_warning);

        self.append(&events_group);
    }

    fn apply_enabled_state(&self, enabled: bool) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        if let Some(switch) = imp.enable_switch.borrow().as_ref() {
            let _guard = switch.freeze_notify();
            switch.set_active(enabled);
        }
        if let Some(scale) = imp.brightness_scale.borrow().as_ref() {
            scale.set_sensitive(enabled);
        }

        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_enabled_after_write_failure(&self) {
        match backend::get_slash_enabled() {
            Ok(enabled) => self.apply_enabled_state(enabled),
            Err(e) => {
                log::error!("Failed to reconcile Slash enabled state after write failure: {e}");
            }
        }
    }

    fn apply_brightness(&self, brightness: u8) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        if let Some(scale) = imp.brightness_scale.borrow().as_ref() {
            let _guard = scale.freeze_notify();
            scale.set_value(brightness as f64);
        }

        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_brightness_after_write_failure(&self) {
        match backend::get_slash_brightness() {
            Ok(brightness) => self.apply_brightness(brightness),
            Err(e) => {
                log::error!("Failed to reconcile Slash brightness after write failure: {e}");
            }
        }
    }

    fn apply_mode(&self, mode: SlashMode) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        if let Some(combo) = imp.mode_combo.borrow().as_ref() {
            let _guard = combo.freeze_notify();
            combo.set_selected(slash_mode_index(mode));
        }

        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_mode_after_write_failure(&self) {
        match backend::get_slash_mode() {
            Ok(mode) => self.apply_mode(mode),
            Err(e) => {
                log::error!("Failed to reconcile Slash mode after write failure: {e}");
            }
        }
    }

    fn apply_interval(&self, interval: u8) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        if let Some(combo) = imp.interval_combo.borrow().as_ref() {
            let _guard = combo.freeze_notify();
            combo.set_selected(interval as u32);
        }

        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_interval_after_write_failure(&self) {
        match backend::get_slash_interval() {
            Ok(interval) => self.apply_interval(interval),
            Err(e) => {
                log::error!("Failed to reconcile Slash interval after write failure: {e}");
            }
        }
    }

    fn apply_event_switch(&self, switch: &adw::SwitchRow, value: bool) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        {
            let _guard = switch.freeze_notify();
            switch.set_active(value);
        }

        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_event_switch(
        &self,
        result: Result<bool, AsusctlError>,
        switch: Option<adw::SwitchRow>,
        setting: &str,
    ) {
        match result {
            Ok(value) => {
                if let Some(switch) = switch {
                    self.apply_event_switch(&switch, value);
                }
            }
            Err(e) => {
                log::error!("Failed to reconcile {setting} after write failure: {e}");
            }
        }
    }

    fn reconcile_show_on_boot_after_write_failure(&self) {
        let switch = self.imp().show_on_boot.borrow().clone();
        self.reconcile_event_switch(
            backend::get_slash_show_on_boot(),
            switch,
            "Slash show-on-boot state",
        );
    }

    fn reconcile_show_on_shutdown_after_write_failure(&self) {
        let switch = self.imp().show_on_shutdown.borrow().clone();
        self.reconcile_event_switch(
            backend::get_slash_show_on_shutdown(),
            switch,
            "Slash show-on-shutdown state",
        );
    }

    fn reconcile_show_on_sleep_after_write_failure(&self) {
        let switch = self.imp().show_on_sleep.borrow().clone();
        self.reconcile_event_switch(
            backend::get_slash_show_on_sleep(),
            switch,
            "Slash show-on-sleep state",
        );
    }

    fn reconcile_show_on_battery_after_write_failure(&self) {
        let switch = self.imp().show_on_battery.borrow().clone();
        self.reconcile_event_switch(
            backend::get_slash_show_on_battery(),
            switch,
            "Slash show-on-battery state",
        );
    }

    fn reconcile_battery_warning_after_write_failure(&self) {
        let switch = self.imp().show_battery_warning.borrow().clone();
        self.reconcile_event_switch(
            backend::get_slash_show_battery_warning(),
            switch,
            "Slash battery-warning state",
        );
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        let imp = self.imp();

        if !*imp.available.borrow() {
            return;
        }

        // Set refreshing flag to prevent signal handlers from firing
        *imp.refreshing.borrow_mut() = true;

        // Load enabled state from D-Bus
        let slash_enabled = if let Some(switch) = imp.enable_switch.borrow().as_ref() {
            match backend::get_slash_enabled() {
                Ok(enabled) => {
                    let _guard = switch.freeze_notify();
                    switch.set_active(enabled);
                    enabled
                }
                Err(e) => {
                    log::error!("Failed to get slash enabled state: {e}");
                    false
                }
            }
        } else {
            false
        };

        // Load brightness from config file and enable/disable based on slash state
        if let Some(scale) = imp.brightness_scale.borrow().as_ref() {
            scale.set_sensitive(slash_enabled);
            match backend::get_slash_brightness() {
                Ok(brightness) => {
                    let _guard = scale.freeze_notify();
                    scale.set_value(brightness as f64);
                }
                Err(e) => {
                    log::error!("Failed to get slash brightness: {e}");
                }
            }
        }

        // Load mode from config file
        if let Some(combo) = imp.mode_combo.borrow().as_ref() {
            match backend::get_slash_mode() {
                Ok(mode) => {
                    let _guard = combo.freeze_notify();
                    combo.set_selected(slash_mode_index(mode));
                }
                Err(e) => {
                    log::error!("Failed to get slash mode: {e}");
                }
            }
        }

        // Load interval from config file
        if let Some(combo) = imp.interval_combo.borrow().as_ref() {
            match backend::get_slash_interval() {
                Ok(interval) => {
                    let _guard = combo.freeze_notify();
                    combo.set_selected(interval as u32);
                }
                Err(e) => {
                    log::error!("Failed to get slash interval: {e}");
                }
            }
        }

        // Load show-on states from D-Bus
        if let Some(switch) = imp.show_on_boot.borrow().as_ref() {
            if let Ok(value) = backend::get_slash_show_on_boot() {
                let _guard = switch.freeze_notify();
                switch.set_active(value);
            }
        }

        if let Some(switch) = imp.show_on_shutdown.borrow().as_ref() {
            if let Ok(value) = backend::get_slash_show_on_shutdown() {
                let _guard = switch.freeze_notify();
                switch.set_active(value);
            }
        }

        if let Some(switch) = imp.show_on_sleep.borrow().as_ref() {
            if let Ok(value) = backend::get_slash_show_on_sleep() {
                let _guard = switch.freeze_notify();
                switch.set_active(value);
            }
        }

        if let Some(switch) = imp.show_on_battery.borrow().as_ref() {
            if let Ok(value) = backend::get_slash_show_on_battery() {
                let _guard = switch.freeze_notify();
                switch.set_active(value);
            }
        }

        if let Some(switch) = imp.show_battery_warning.borrow().as_ref() {
            if let Ok(value) = backend::get_slash_show_battery_warning() {
                let _guard = switch.freeze_notify();
                switch.set_active(value);
            }
        }

        // Clear refreshing flag
        *imp.refreshing.borrow_mut() = false;
    }
}

fn slash_mode_index(mode: SlashMode) -> u32 {
    match mode {
        SlashMode::Static => 0,
        SlashMode::Bounce => 1,
        SlashMode::Slash => 2,
        SlashMode::Loading => 3,
        SlashMode::BitStream => 4,
        SlashMode::Transmission => 5,
        SlashMode::Flow => 6,
        SlashMode::Flux => 7,
        SlashMode::Phantom => 8,
        SlashMode::Spectrum => 9,
        SlashMode::Hazard => 10,
        SlashMode::Interfacing => 11,
        SlashMode::Ramp => 12,
        SlashMode::GameOver => 13,
        SlashMode::Start => 14,
        SlashMode::Buzzer => 15,
    }
}

impl Default for SlashPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Refreshable for SlashPage {
    fn refresh(&self) {
        self.refresh_data();
    }
}
