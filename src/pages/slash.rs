use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SlashPage;

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

        power_group.add(&enable_row);
        self.append(&power_group);

        // Brightness group
        let brightness_group = adw::PreferencesGroup::builder().title("Brightness").build();

        let brightness_row = adw::ActionRow::builder().title("Brightness Level").build();

        let brightness_scale = gtk4::Scale::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .adjustment(&gtk4::Adjustment::new(50.0, 0.0, 100.0, 5.0, 10.0, 0.0))
            .width_request(200)
            .valign(gtk4::Align::Center)
            .draw_value(true)
            .build();

        brightness_row.add_suffix(&brightness_scale);
        brightness_group.add(&brightness_row);

        self.append(&brightness_group);

        // Mode group
        let mode_group = adw::PreferencesGroup::builder()
            .title("Animation Mode")
            .build();

        let modes = [
            ("Static", "Solid lighting"),
            ("Breathe", "Pulsing effect"),
            ("Strobe", "Flashing effect"),
            ("Rainbow", "Color cycle"),
            ("Bounce", "Bouncing animation"),
            ("Loading", "Loading bar animation"),
            ("Slash", "Slash animation"),
        ];

        for (mode, description) in modes {
            let row = adw::ActionRow::builder()
                .title(mode)
                .subtitle(description)
                .activatable(true)
                .build();

            let checkmark = gtk4::Image::from_icon_name("object-select-symbolic");
            checkmark.set_visible(false);
            row.add_suffix(&checkmark);

            mode_group.add(&row);
        }

        self.append(&mode_group);

        // Interval group
        let interval_group = adw::PreferencesGroup::builder()
            .title("Animation Speed")
            .build();

        let interval_row = adw::ComboRow::builder()
            .title("Speed")
            .subtitle("Animation interval")
            .model(&gtk4::StringList::new(&["Slow", "Medium", "Fast"]))
            .selected(1)
            .build();

        interval_group.add(&interval_row);
        self.append(&interval_group);
    }
}

impl Default for SlashPage {
    fn default() -> Self {
        Self::new()
    }
}
