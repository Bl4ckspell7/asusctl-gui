use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, SlashMode};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SlashPage {
        pub enable_switch: RefCell<Option<adw::SwitchRow>>,
        pub brightness_scale: RefCell<Option<gtk4::Scale>>,
        pub mode_combo: RefCell<Option<adw::ComboRow>>,
        pub interval_combo: RefCell<Option<adw::ComboRow>>,
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
            self.obj().load_data();
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
        let power_group = adw::PreferencesGroup::builder()
            .title("Power")
            .build();

        let enable_row = adw::SwitchRow::builder()
            .title("Enable Slash Lighting")
            .subtitle("Turn the LED bar on or off")
            .build();

        // Connect the switch to enable/disable slash
        enable_row.connect_active_notify(|switch| {
            let result = if switch.is_active() {
                backend::enable_slash()
            } else {
                backend::disable_slash()
            };

            if let Err(e) = result {
                eprintln!("Failed to toggle slash: {}", e);
            }
        });

        imp.enable_switch.replace(Some(enable_row.clone()));
        power_group.add(&enable_row);
        self.append(&power_group);

        // Brightness group
        let brightness_group = adw::PreferencesGroup::builder()
            .title("Brightness")
            .build();

        let brightness_row = adw::ActionRow::builder()
            .title("Brightness Level")
            .subtitle("0-255")
            .build();

        let brightness_scale = gtk4::Scale::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .adjustment(&gtk4::Adjustment::new(128.0, 0.0, 255.0, 1.0, 10.0, 0.0))
            .width_request(200)
            .valign(gtk4::Align::Center)
            .draw_value(true)
            .build();

        // Connect brightness scale to set slash brightness
        brightness_scale.connect_value_changed(|scale| {
            let value = scale.value() as u8;
            if let Err(e) = backend::set_slash_brightness(value) {
                eprintln!("Failed to set slash brightness: {}", e);
            }
        });

        imp.brightness_scale.replace(Some(brightness_scale.clone()));
        brightness_row.add_suffix(&brightness_scale);
        brightness_group.add(&brightness_row);

        self.append(&brightness_group);

        // Mode group
        let mode_group = adw::PreferencesGroup::builder()
            .title("Animation")
            .build();

        // Create mode names list for combo
        let mode_names: Vec<&str> = SLASH_MODES.iter().map(|(name, _)| *name).collect();
        let mode_combo = adw::ComboRow::builder()
            .title("Mode")
            .subtitle("Animation style")
            .model(&gtk4::StringList::new(&mode_names))
            .build();

        // Connect mode combo to set slash mode
        mode_combo.connect_selected_notify(|combo| {
            let mode = match combo.selected() {
                0 => SlashMode::Bounce,
                1 => SlashMode::Slash,
                2 => SlashMode::Loading,
                3 => SlashMode::BitStream,
                4 => SlashMode::Transmission,
                5 => SlashMode::Flow,
                6 => SlashMode::Flux,
                7 => SlashMode::Phantom,
                8 => SlashMode::Spectrum,
                9 => SlashMode::Hazard,
                10 => SlashMode::Interfacing,
                11 => SlashMode::Ramp,
                12 => SlashMode::GameOver,
                13 => SlashMode::Start,
                14 => SlashMode::Buzzer,
                _ => return,
            };

            if let Err(e) = backend::set_slash_mode(mode) {
                eprintln!("Failed to set slash mode: {}", e);
            }
        });

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
        interval_combo.connect_selected_notify(|combo| {
            let interval = combo.selected() as u8;
            if let Err(e) = backend::set_slash_interval(interval) {
                eprintln!("Failed to set slash interval: {}", e);
            }
        });

        imp.interval_combo.replace(Some(interval_combo.clone()));
        mode_group.add(&interval_combo);
        self.append(&mode_group);
    }

    fn load_data(&self) {
        let imp = self.imp();

        // Load enabled state from config file
        if let Some(switch) = imp.enable_switch.borrow().as_ref() {
            match backend::get_slash_enabled() {
                Ok(enabled) => {
                    switch.set_active(enabled);
                }
                Err(e) => {
                    eprintln!("Failed to get slash enabled state: {}", e);
                }
            }
        }

        // Load brightness from config file
        if let Some(scale) = imp.brightness_scale.borrow().as_ref() {
            match backend::get_slash_brightness() {
                Ok(brightness) => {
                    scale.set_value(brightness as f64);
                }
                Err(e) => {
                    eprintln!("Failed to get slash brightness: {}", e);
                }
            }
        }

        // Load mode from config file
        if let Some(combo) = imp.mode_combo.borrow().as_ref() {
            match backend::get_slash_mode() {
                Ok(mode) => {
                    let index = match mode {
                        SlashMode::Bounce => 0,
                        SlashMode::Slash => 1,
                        SlashMode::Loading => 2,
                        SlashMode::BitStream => 3,
                        SlashMode::Transmission => 4,
                        SlashMode::Flow => 5,
                        SlashMode::Flux => 6,
                        SlashMode::Phantom => 7,
                        SlashMode::Spectrum => 8,
                        SlashMode::Hazard => 9,
                        SlashMode::Interfacing => 10,
                        SlashMode::Ramp => 11,
                        SlashMode::GameOver => 12,
                        SlashMode::Start => 13,
                        SlashMode::Buzzer => 14,
                    };
                    combo.set_selected(index);
                }
                Err(e) => {
                    eprintln!("Failed to get slash mode: {}", e);
                }
            }
        }

        // Load interval from config file
        if let Some(combo) = imp.interval_combo.borrow().as_ref() {
            match backend::get_slash_interval() {
                Ok(interval) => {
                    combo.set_selected(interval as u32);
                }
                Err(e) => {
                    eprintln!("Failed to get slash interval: {}", e);
                }
            }
        }
    }
}

impl Default for SlashPage {
    fn default() -> Self {
        Self::new()
    }
}
