use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::glib;
use libadwaita as adw;
use adw::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AuraPage;

    #[glib::object_subclass]
    impl ObjectSubclass for AuraPage {
        const NAME: &'static str = "AuraPage";
        type Type = super::AuraPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for AuraPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for AuraPage {}
    impl BoxImpl for AuraPage {}
}

glib::wrapper! {
    pub struct AuraPage(ObjectSubclass<imp::AuraPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl AuraPage {
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
            .label("Aura Lighting")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&title);

        // Keyboard brightness group
        let brightness_group = adw::PreferencesGroup::builder()
            .title("Keyboard Brightness")
            .build();

        let brightness_row = adw::ActionRow::builder()
            .title("Brightness Level")
            .build();

        // Brightness buttons
        let brightness_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .build();

        for level in ["Off", "Low", "Med", "High"] {
            let btn = gtk4::Button::builder()
                .label(level)
                .css_classes(["flat"])
                .build();
            brightness_box.append(&btn);
        }

        brightness_row.add_suffix(&brightness_box);
        brightness_group.add(&brightness_row);

        self.append(&brightness_group);

        // Lighting mode group
        let mode_group = adw::PreferencesGroup::builder()
            .title("Lighting Mode")
            .build();

        let modes = [
            ("Static", "Single color"),
            ("Breathe", "Pulsing effect"),
            ("Rainbow", "Color cycle"),
            ("Star", "Twinkling effect"),
            ("Rain", "Falling drops"),
            ("Highlight", "Reactive typing"),
            ("Laser", "Laser effect"),
            ("Ripple", "Ripple on keypress"),
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

        // Color selection group
        let color_group = adw::PreferencesGroup::builder()
            .title("Color")
            .build();

        let color_row = adw::ActionRow::builder()
            .title("Lighting Color")
            .subtitle("Select keyboard color")
            .build();

        let color_dialog = gtk4::ColorDialog::builder().build();
        let color_button = gtk4::ColorDialogButton::builder()
            .dialog(&color_dialog)
            .valign(gtk4::Align::Center)
            .build();

        color_row.add_suffix(&color_button);
        color_row.set_activatable_widget(Some(&color_button));
        color_group.add(&color_row);

        self.append(&color_group);
    }
}

impl Default for AuraPage {
    fn default() -> Self {
        Self::new()
    }
}
