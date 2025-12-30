use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ProfilePage;

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
            ("Quiet", "power-profile-power-saver-symbolic", "Reduced fan noise, lower performance"),
            ("Balanced", "power-profile-balanced-symbolic", "Balance between performance and noise"),
            ("Performance", "power-profile-performance-symbolic", "Maximum performance"),
        ];

        // Create first radio button as the group leader
        let mut first_radio: Option<gtk4::CheckButton> = None;

        for (name, icon, description) in profiles {
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

            row.add_suffix(&radio);
            row.set_activatable_widget(Some(&radio));

            current_group.add(&row);
        }

        self.append(&current_group);

        // AC power profile group
        let ac_group = adw::PreferencesGroup::builder()
            .title("On AC Power")
            .description("Profile to use when connected to power")
            .build();

        let ac_combo = adw::ComboRow::builder()
            .title("Power Profile")
            .model(&gtk4::StringList::new(&["Quiet", "Balanced", "Performance"]))
            .selected(2) // Performance by default on AC
            .build();

        ac_group.add(&ac_combo);
        self.append(&ac_group);

        // Battery profile group
        let battery_group = adw::PreferencesGroup::builder()
            .title("On Battery")
            .description("Profile to use when on battery power")
            .build();

        let battery_combo = adw::ComboRow::builder()
            .title("Power Profile")
            .model(&gtk4::StringList::new(&["Quiet", "Balanced", "Performance"]))
            .selected(0) // Quiet by default on battery
            .build();

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

        charge_limit_row.add_suffix(&charge_scale);
        battery_settings.add(&charge_limit_row);

        self.append(&battery_settings);
    }
}

impl Default for ProfilePage {
    fn default() -> Self {
        Self::new()
    }
}
