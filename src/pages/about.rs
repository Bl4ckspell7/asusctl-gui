use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AboutPage;

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
            .subtitle("ASUS ROG Zephyrus G14")
            .build();

        let driver_row = adw::ActionRow::builder()
            .title("Armoury Crate Driver")
            .subtitle("Checking...")
            .build();

        let asusctl_row = adw::ActionRow::builder()
            .title("asusctl Version")
            .subtitle("Checking...")
            .build();

        laptop_group.add(&model_row);
        laptop_group.add(&driver_row);
        laptop_group.add(&asusctl_row);

        self.append(&laptop_group);

        // Supported features group
        let features_group = adw::PreferencesGroup::builder()
            .title("Supported Features")
            .build();

        let placeholder = adw::ActionRow::builder()
            .title("Features will be listed here")
            .subtitle("Run 'asusctl --show-supported' to check")
            .build();

        features_group.add(&placeholder);

        self.append(&features_group);
    }
}

impl Default for AboutPage {
    fn default() -> Self {
        Self::new()
    }
}
