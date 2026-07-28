use adw::prelude::*;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend;
use crate::ui::Refreshable;

/// Placeholder shown for the serial number until the user reveals it.
const SERIAL_HIDDEN: &str = "[ Hidden ]";
/// Shown while the polkit prompt is open.
const SERIAL_PENDING: &str = "Requesting permission...";
/// Shown when the firmware has no serial to reveal.
const SERIAL_MISSING: &str = "Not set";

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AboutPage {
        pub model_row: RefCell<Option<adw::ActionRow>>,
        pub driver_row: RefCell<Option<adw::ActionRow>>,
        pub asusctl_row: RefCell<Option<adw::ActionRow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AboutPage {
        const NAME: &'static str = "AboutPage";
        type Type = super::AboutPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for AboutPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().refresh_data();
        }
    }

    impl WidgetImpl for AboutPage {}
    impl BoxImpl for AboutPage {}
}

glib::wrapper! {
    pub struct AboutPage(ObjectSubclass<imp::AboutPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl AboutPage {
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
            .label("About")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&title);

        // Laptop info group
        let laptop_group = adw::PreferencesGroup::builder()
            .title("Laptop Information")
            .build();

        let model_row = adw::ActionRow::builder()
            .title("Model")
            .subtitle("Loading...")
            .build();

        let driver_row = adw::ActionRow::builder()
            .title("Board Name")
            .subtitle("Loading...")
            .build();

        // DMI values are static for the boot, so they are read once here rather
        // than on every refresh.
        let dmi = backend::get_dmi_info();

        let board_vendor_row = adw::ActionRow::builder()
            .title("Board Vendor")
            .subtitle(Self::or_unknown(&dmi.board_vendor))
            .build();

        let serial_row = Self::build_serial_row();

        let asusctl_row = adw::ActionRow::builder()
            .title("asusctl Version")
            .subtitle("Loading...")
            .build();

        let distro_row = adw::ActionRow::builder()
            .title("Distribution")
            .subtitle(&backend::get_distro())
            .build();

        let kernel_row = adw::ActionRow::builder()
            .title("Kernel Version")
            .subtitle(&backend::get_kernel_version())
            .build();

        laptop_group.add(&model_row);
        laptop_group.add(&driver_row);
        laptop_group.add(&board_vendor_row);
        laptop_group.add(&serial_row);
        laptop_group.add(&asusctl_row);
        laptop_group.add(&distro_row);
        laptop_group.add(&kernel_row);

        // Store references
        imp.model_row.replace(Some(model_row));
        imp.driver_row.replace(Some(driver_row));
        imp.asusctl_row.replace(Some(asusctl_row));

        self.append(&laptop_group);

        // Firmware group
        let firmware_group = adw::PreferencesGroup::builder().title("Firmware").build();

        let bios_row = adw::ActionRow::builder()
            .title("BIOS")
            .subtitle(backend::format_bios(dmi))
            .build();

        let bios_vendor_row = adw::ActionRow::builder()
            .title("BIOS Vendor")
            .subtitle(Self::or_unknown(&dmi.bios_vendor))
            .build();

        firmware_group.add(&bios_row);
        firmware_group.add(&bios_vendor_row);

        self.append(&firmware_group);

        // Service status group
        let status_group = adw::PreferencesGroup::builder()
            .title("Service Status")
            .build();

        // Supported features group
        let features_group = adw::PreferencesGroup::builder()
            .title("Supported Features")
            .build();

        let features = backend::detect_features();
        Self::populate_status(&status_group, features);
        Self::populate_features(&features_group, features);

        self.append(&status_group);
        self.append(&features_group);
    }

    /// Fall back to "Unknown" for DMI fields the firmware left empty.
    fn or_unknown(value: &str) -> &str {
        if value.is_empty() { "Unknown" } else { value }
    }

    /// Build the serial number row. The value stays hidden until the user asks
    /// for it, because `/sys/class/dmi/id/product_serial` is root-only and
    /// reading it triggers a polkit prompt.
    fn build_serial_row() -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title("Serial Number")
            .subtitle(SERIAL_HIDDEN)
            .build();

        let button = gtk4::Button::builder()
            .label("Reveal")
            .valign(gtk4::Align::Center)
            .css_classes(["flat"])
            .build();

        let row_weak = row.downgrade();
        button.connect_clicked(move |button| {
            let Some(row) = row_weak.upgrade() else {
                return;
            };

            button.set_sensitive(false);
            row.set_subtitle(SERIAL_PENDING);

            // pkexec blocks until the polkit dialog is answered, so it must not
            // run on the main loop.
            let row_weak = row.downgrade();
            let button_weak = button.downgrade();
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(backend::read_serial_number).await;

                let Some(row) = row_weak.upgrade() else {
                    return;
                };

                let error = match result {
                    Ok(Ok(serial)) => {
                        // A firmware without a serial is a final answer, so the
                        // button goes away either way.
                        row.set_subtitle(serial.as_deref().unwrap_or(SERIAL_MISSING));
                        if let Some(button) = button_weak.upgrade() {
                            button.set_visible(false);
                        }
                        return;
                    }
                    Ok(Err(e)) => e.to_string(),
                    Err(_) => "the reading thread panicked".to_string(),
                };

                // Recoverable: the user can dismiss a polkit prompt and retry.
                log::warn!("Failed to read serial number: {error}");
                row.set_subtitle(SERIAL_HIDDEN);
                if let Some(button) = button_weak.upgrade() {
                    button.set_sensitive(true);
                }
            });
        });

        row.add_suffix(&button);
        row
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        let imp = self.imp();

        // Load system info
        match backend::get_system_info() {
            Ok(info) => {
                if let Some(row) = imp.model_row.borrow().as_ref() {
                    row.set_subtitle(&info.product_family);
                }
                if let Some(row) = imp.driver_row.borrow().as_ref() {
                    row.set_subtitle(&info.board_name);
                }
                if let Some(row) = imp.asusctl_row.borrow().as_ref() {
                    row.set_subtitle(&format!("v{}", info.asusctl_version));
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                if let Some(row) = imp.model_row.borrow().as_ref() {
                    row.set_subtitle(&error_msg);
                }
                if let Some(row) = imp.driver_row.borrow().as_ref() {
                    row.set_subtitle(&error_msg);
                }
                if let Some(row) = imp.asusctl_row.borrow().as_ref() {
                    row.set_subtitle(&error_msg);
                }
            }
        }
    }

    fn populate_status(group: &adw::PreferencesGroup, features: &backend::SupportedFeatures) {
        let service_status = [
            ("Installed", "asusctl", features.asusctl_installed),
            ("Service Running", "asusd", features.asusd_running),
            ("Driver Loaded", "asus-armoury", features.has_armoury),
        ];

        for (title, subtitle, available) in service_status {
            let row = adw::ActionRow::builder()
                .title(title)
                .subtitle(subtitle)
                .build();

            let icon_name = if available {
                "object-select-symbolic"
            } else {
                "window-close-symbolic"
            };

            let icon = gtk4::Image::from_icon_name(icon_name);
            if available {
                icon.add_css_class("success");
            } else {
                icon.add_css_class("error");
            }
            row.add_suffix(&icon);

            group.add(&row);
        }
    }

    fn populate_features(group: &adw::PreferencesGroup, features: &backend::SupportedFeatures) {
        // Core features
        let core_features = [
            ("Aura (Keyboard Lighting)", features.has_aura),
            ("Platform Control", features.has_platform),
            ("Fan Curves", features.has_fan_curves),
            ("Slash (LED Bar)", features.has_slash),
        ];

        for (name, supported) in core_features {
            let row = adw::ActionRow::builder().title(name).build();

            let icon_name = if supported {
                "object-select-symbolic"
            } else {
                "window-close-symbolic"
            };

            let icon = gtk4::Image::from_icon_name(icon_name);
            if supported {
                icon.add_css_class("success");
            } else {
                icon.add_css_class("error");
            }
            row.add_suffix(&icon);

            group.add(&row);
        }

        // Platform properties
        let platform_props = [
            ("Charge Control", features.has_charge_control),
            ("Throttle Policy", features.has_throttle_policy),
        ];

        for (name, supported) in platform_props {
            let row = adw::ActionRow::builder().title(name).build();

            let icon_name = if supported {
                "object-select-symbolic"
            } else {
                "window-close-symbolic"
            };

            let icon = gtk4::Image::from_icon_name(icon_name);
            if supported {
                icon.add_css_class("success");
            } else {
                icon.add_css_class("error");
            }
            row.add_suffix(&icon);

            group.add(&row);
        }

        // Keyboard brightness levels
        if !features.keyboard_brightness_levels.is_empty() {
            let levels: Vec<String> = features
                .keyboard_brightness_levels
                .iter()
                .map(|l| format!("{l}"))
                .collect();

            let row = adw::ActionRow::builder()
                .title("Keyboard Brightness Levels")
                .subtitle(&levels.join(", "))
                .build();

            group.add(&row);
        }

        // Aura modes
        if !features.aura_modes.is_empty() {
            let modes: Vec<String> = features.aura_modes.iter().map(|m| format!("{m}")).collect();

            let row = adw::ActionRow::builder()
                .title("Aura Modes")
                .subtitle(&modes.join(", "))
                .build();

            group.add(&row);
        }
    }
}

impl Default for AboutPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Refreshable for AboutPage {
    fn refresh(&self) {
        self.refresh_data();
    }
}
