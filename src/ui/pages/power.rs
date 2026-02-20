use adw::prelude::*;
use gtk4::glib;
use gtk4::glib::prelude::ObjectExt;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, PowerProfile};
use crate::ui::Refreshable;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PowerPage {
        pub available: RefCell<bool>,
        pub charge_control_available: RefCell<bool>,
        pub profile_radios: RefCell<Vec<gtk4::CheckButton>>,
        pub ac_combo: RefCell<Option<adw::ComboRow>>,
        pub battery_combo: RefCell<Option<adw::ComboRow>>,
        pub charge_scale: RefCell<Option<gtk4::Scale>>,
        pub refreshing: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PowerPage {
        const NAME: &'static str = "PowerPage";
        type Type = super::PowerPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for PowerPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().refresh_data();
        }
    }

    impl WidgetImpl for PowerPage {}
    impl BoxImpl for PowerPage {}
}

glib::wrapper! {
    pub struct PowerPage(ObjectSubclass<imp::PowerPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl PowerPage {
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

        if !features.asusctl_installed {
            *imp.available.borrow_mut() = false;
            let status = adw::StatusPage::builder()
                .icon_name("gnome-power-manager-symbolic")
                .title("Power Profiles Unavailable")
                .description("asusctl is not installed. Install asusctl to manage power profiles and charge control.")
                .vexpand(true)
                .build();
            self.append(&status);
            return;
        }

        *imp.available.borrow_mut() = true;
        *imp.charge_control_available.borrow_mut() = features.has_charge_control;

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
            let this = self.clone();
            radio.connect_toggled(move |button| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if button.is_active() {
                    if let Err(e) = backend::set_profile(profile_clone) {
                        log::error!("Failed to set profile: {e}");
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

        // Connect AC combo to set profile on AC power
        {
            let this = self.clone();
            ac_combo.connect_selected_notify(move |combo| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let profile = match combo.selected() {
                    0 => PowerProfile::Quiet,
                    1 => PowerProfile::Balanced,
                    _ => PowerProfile::Performance,
                };
                if let Err(e) = backend::set_profile_ac(profile) {
                    log::error!("Failed to set AC profile: {e}");
                }
            });
        }

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

        // Connect battery combo to set profile on battery power
        {
            let this = self.clone();
            battery_combo.connect_selected_notify(move |combo| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let profile = match combo.selected() {
                    0 => PowerProfile::Quiet,
                    1 => PowerProfile::Balanced,
                    _ => PowerProfile::Performance,
                };
                if let Err(e) = backend::set_profile_battery(profile) {
                    log::error!("Failed to set battery profile: {e}");
                }
            });
        }

        imp.battery_combo.replace(Some(battery_combo.clone()));
        battery_group.add(&battery_combo);
        self.append(&battery_group);

        // Battery settings group (only if charge control is supported)
        if features.has_charge_control {
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
            {
                let this = self.clone();
                charge_scale.connect_value_changed(move |scale| {
                    if *this.imp().refreshing.borrow() {
                        return;
                    }
                    let value = scale.value() as u8;
                    if let Err(e) = backend::set_charge_limit(value) {
                        log::error!("Failed to set charge limit: {e}");
                    }
                });
            }

            imp.charge_scale.replace(Some(charge_scale.clone()));
            charge_limit_row.add_suffix(&charge_scale);
            battery_settings.add(&charge_limit_row);

            self.append(&battery_settings);
        }
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        let imp = self.imp();

        if !*imp.available.borrow() {
            return;
        }

        *imp.refreshing.borrow_mut() = true;

        // Get current profile state via CLI (more reliable mapping)
        match backend::get_profile_state() {
            Ok(state) => {
                // Update profile radios
                {
                    let radios = imp.profile_radios.borrow();
                    let index = match state.active {
                        PowerProfile::Quiet => 0,
                        PowerProfile::Balanced => 1,
                        PowerProfile::Performance => 2,
                    };

                    // Freeze all radios to prevent GTK state accounting issues
                    // Guards auto-call thaw_notify when dropped
                    let _guards: Vec<_> =
                        radios.iter().map(|radio| radio.freeze_notify()).collect();
                    if let Some(radio) = radios.get(index) {
                        radio.set_active(true);
                    }
                }

                // Set AC combo
                if let Some(combo) = imp.ac_combo.borrow().as_ref() {
                    let _guard = combo.freeze_notify();
                    let ac_index = match state.on_ac {
                        PowerProfile::Quiet => 0,
                        PowerProfile::Balanced => 1,
                        PowerProfile::Performance => 2,
                    };
                    combo.set_selected(ac_index);
                }

                // Set battery combo
                if let Some(combo) = imp.battery_combo.borrow().as_ref() {
                    let _guard = combo.freeze_notify();
                    let bat_index = match state.on_battery {
                        PowerProfile::Quiet => 0,
                        PowerProfile::Balanced => 1,
                        PowerProfile::Performance => 2,
                    };
                    combo.set_selected(bat_index);
                }
            }
            Err(e) => {
                log::error!("Failed to get profile state: {e}");
            }
        }

        // Load charge limit via D-Bus (only if charge control is supported)
        if *imp.charge_control_available.borrow() {
            if let Some(scale) = imp.charge_scale.borrow().as_ref() {
                match backend::get_charge_limit_dbus() {
                    Ok(limit) => {
                        let _guard = scale.freeze_notify();
                        scale.set_value(limit as f64);
                    }
                    Err(e) => {
                        log::error!("Failed to get charge limit: {e}");
                    }
                }
            }
        }

        *imp.refreshing.borrow_mut() = false;
    }
}

impl Default for PowerPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Refreshable for PowerPage {
    fn refresh(&self) {
        self.refresh_data();
    }
}
