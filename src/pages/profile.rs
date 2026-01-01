use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, PowerProfile};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ProfilePage {
        pub profile_radios: RefCell<Vec<gtk4::CheckButton>>,
        pub ac_combo: RefCell<Option<adw::ComboRow>>,
        pub battery_combo: RefCell<Option<adw::ComboRow>>,
        pub charge_scale: RefCell<Option<gtk4::Scale>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProfilePage {
        const NAME: &'static str = "ProfilePage";
        type Type = super::ProfilePage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for ProfilePage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().load_data();
        }
    }

    impl WidgetImpl for ProfilePage {}
    impl BoxImpl for ProfilePage {}
}

glib::wrapper! {
    pub struct ProfilePage(ObjectSubclass<imp::ProfilePage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl ProfilePage {
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
            .label("Power Profiles")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&title);

        // Current profile group
        let current_group = adw::PreferencesGroup::builder()
            .title("Current Profile")
            .build();

        let profiles = [
            (
                PowerProfile::Quiet,
                "Quiet",
                "power-profile-power-saver-symbolic",
                "Reduced fan noise, lower performance",
            ),
            (
                PowerProfile::Balanced,
                "Balanced",
                "power-profile-balanced-symbolic",
                "Balance between performance and noise",
            ),
            (
                PowerProfile::Performance,
                "Performance",
                "power-profile-performance-symbolic",
                "Maximum performance",
            ),
        ];

        let mut radios: Vec<gtk4::CheckButton> = Vec::new();
        let mut first_radio: Option<gtk4::CheckButton> = None;

        for (profile, name, icon, description) in profiles {
            let row = adw::ActionRow::builder()
                .title(name)
                .subtitle(description)
                .activatable(true)
                .build();

            let icon_widget = gtk4::Image::from_icon_name(icon);
            row.add_prefix(&icon_widget);

            let radio = gtk4::CheckButton::builder()
                .valign(gtk4::Align::Center)
                .build();

            // Set the group for radio button behavior
            if let Some(ref group) = first_radio {
                radio.set_group(Some(group));
            } else {
                first_radio = Some(radio.clone());
            }

            // Connect toggled handler to set profile
            let profile_clone = profile;
            radio.connect_toggled(move |button| {
                if button.is_active() {
                    if let Err(e) = backend::set_profile(profile_clone) {
                        eprintln!("Failed to set profile: {}", e);
                    }
                }
            });

            row.add_suffix(&radio);
            row.set_activatable_widget(Some(&radio));

            radios.push(radio);
            current_group.add(&row);
        }

        imp.profile_radios.replace(radios);

        self.append(&current_group);

        // AC power profile group
        let ac_group = adw::PreferencesGroup::builder()
            .title("On AC Power")
            .description("Profile to use when connected to power")
            .build();

        let ac_combo = adw::ComboRow::builder()
            .title("Power Profile")
            .model(&gtk4::StringList::new(&[
                "Quiet",
                "Balanced",
                "Performance",
            ]))
            .selected(2) // Performance by default on AC
            .build();

        imp.ac_combo.replace(Some(ac_combo.clone()));
        ac_group.add(&ac_combo);
        self.append(&ac_group);

        // Battery profile group
        let battery_group = adw::PreferencesGroup::builder()
            .title("On Battery")
            .description("Profile to use when on battery power")
            .build();

        let battery_combo = adw::ComboRow::builder()
            .title("Power Profile")
            .model(&gtk4::StringList::new(&[
                "Quiet",
                "Balanced",
                "Performance",
            ]))
            .selected(0) // Quiet by default on battery
            .build();

        imp.battery_combo.replace(Some(battery_combo.clone()));
        battery_group.add(&battery_combo);
        self.append(&battery_group);

        // Battery settings group
        let battery_settings = adw::PreferencesGroup::builder()
            .title("Battery Settings")
            .build();

        let charge_limit_row = adw::ActionRow::builder()
            .title("Charge Limit")
            .subtitle("Limit maximum charge to extend battery lifespan")
            .build();

        let charge_scale = gtk4::Scale::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .adjustment(&gtk4::Adjustment::new(80.0, 20.0, 100.0, 5.0, 10.0, 0.0))
            .width_request(200)
            .valign(gtk4::Align::Center)
            .draw_value(true)
            .build();

        // Connect charge scale to set charge limit
        charge_scale.connect_value_changed(|scale| {
            let value = scale.value() as u8;
            if let Err(e) = backend::set_charge_limit(value) {
                eprintln!("Failed to set charge limit: {}", e);
            }
        });

        imp.charge_scale.replace(Some(charge_scale.clone()));
        charge_limit_row.add_suffix(&charge_scale);
        battery_settings.add(&charge_limit_row);

        self.append(&battery_settings);
    }

    fn load_data(&self) {
        let imp = self.imp();

        // Get current profile state via CLI (more reliable mapping)
        match backend::get_profile_state() {
            Ok(state) => {
                let radios = imp.profile_radios.borrow();
                let index = match state.active {
                    PowerProfile::Quiet => 0,
                    PowerProfile::Balanced => 1,
                    PowerProfile::Performance => 2,
                };

                if let Some(radio) = radios.get(index) {
                    radio.set_active(true);
                }

                // Set AC combo
                if let Some(combo) = imp.ac_combo.borrow().as_ref() {
                    let ac_index = match state.on_ac {
                        PowerProfile::Quiet => 0,
                        PowerProfile::Balanced => 1,
                        PowerProfile::Performance => 2,
                    };
                    combo.set_selected(ac_index);
                }

                // Set battery combo
                if let Some(combo) = imp.battery_combo.borrow().as_ref() {
                    let bat_index = match state.on_battery {
                        PowerProfile::Quiet => 0,
                        PowerProfile::Balanced => 1,
                        PowerProfile::Performance => 2,
                    };
                    combo.set_selected(bat_index);
                }
            }
            Err(e) => {
                eprintln!("Failed to get profile state: {}", e);
            }
        }

        // Load charge limit via D-Bus
        if let Some(scale) = imp.charge_scale.borrow().as_ref() {
            match backend::get_charge_limit_dbus() {
                Ok(limit) => {
                    scale.set_value(limit as f64);
                }
                Err(e) => {
                    eprintln!("Failed to get charge limit: {}", e);
                }
            }
        }
    }
}

impl Default for ProfilePage {
    fn default() -> Self {
        Self::new()
    }
}
