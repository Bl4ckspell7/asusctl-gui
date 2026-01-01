use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AboutPage {
        pub model_row: RefCell<Option<adw::ActionRow>>,
        pub driver_row: RefCell<Option<adw::ActionRow>>,
        pub asusctl_row: RefCell<Option<adw::ActionRow>>,
        pub features_group: RefCell<Option<adw::PreferencesGroup>>,
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
            self.obj().load_data();
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

        let asusctl_row = adw::ActionRow::builder()
            .title("asusctl Version")
            .subtitle("Loading...")
            .build();

        laptop_group.add(&model_row);
        laptop_group.add(&driver_row);
        laptop_group.add(&asusctl_row);

        // Store references
        imp.model_row.replace(Some(model_row));
        imp.driver_row.replace(Some(driver_row));
        imp.asusctl_row.replace(Some(asusctl_row));

        self.append(&laptop_group);

        // Supported features group
        let features_group = adw::PreferencesGroup::builder()
            .title("Supported Features")
            .build();

        imp.features_group.replace(Some(features_group.clone()));

        self.append(&features_group);
    }

    fn load_data(&self) {
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

        // Load supported features
        if let Some(features_group) = imp.features_group.borrow().as_ref() {
            match backend::get_supported_features() {
                Ok(features) => {
                    self.populate_features(features_group, &features);
                }
                Err(e) => {
                    let error_row = adw::ActionRow::builder()
                        .title("Error loading features")
                        .subtitle(&e.to_string())
                        .build();
                    features_group.add(&error_row);
                }
            }
        }
    }

    fn populate_features(&self, group: &adw::PreferencesGroup, features: &backend::SupportedFeatures) {
        // Core features
        let core_features = [
            ("Aura (Keyboard Lighting)", features.has_aura),
            ("Platform Control", features.has_platform),
            ("Fan Curves", features.has_fan_curves),
            ("Slash (LED Bar)", features.has_slash),
        ];

        for (name, supported) in core_features {
            let row = adw::ActionRow::builder()
                .title(name)
                .build();

            let icon_name = if supported {
                "emblem-ok-symbolic"
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
            let row = adw::ActionRow::builder()
                .title(name)
                .build();

            let icon_name = if supported {
                "emblem-ok-symbolic"
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
                .map(|l| format!("{}", l))
                .collect();

            let row = adw::ActionRow::builder()
                .title("Keyboard Brightness Levels")
                .subtitle(&levels.join(", "))
                .build();

            group.add(&row);
        }

        // Aura modes
        if !features.aura_modes.is_empty() {
            let modes: Vec<String> = features.aura_modes.iter().map(|m| format!("{}", m)).collect();

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
