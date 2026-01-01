use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, KeyboardBrightness};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AuraPage {
        pub brightness_buttons: RefCell<Vec<gtk4::ToggleButton>>,
    }

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
            self.obj().load_data();
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
        let imp = self.imp();

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

        let brightness_row = adw::ActionRow::builder().title("Brightness Level").build();

        // Brightness toggle buttons (linked group)
        let brightness_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["linked"])
            .valign(gtk4::Align::Center)
            .build();

        let levels = [
            (KeyboardBrightness::Off, "Off"),
            (KeyboardBrightness::Low, "Low"),
            (KeyboardBrightness::Med, "Med"),
            (KeyboardBrightness::High, "High"),
        ];

        let mut buttons: Vec<gtk4::ToggleButton> = Vec::new();

        for (level, label) in levels {
            let btn = gtk4::ToggleButton::builder().label(label).build();

            // Connect click handler to set brightness
            let level_clone = level;
            btn.connect_clicked(move |button| {
                if button.is_active() {
                    if let Err(e) = backend::set_keyboard_brightness(level_clone) {
                        eprintln!("Failed to set brightness: {}", e);
                    }
                }
            });

            brightness_box.append(&btn);
            buttons.push(btn);
        }

        // Link buttons together so only one can be active
        for i in 1..buttons.len() {
            buttons[i].set_group(Some(&buttons[0]));
        }

        imp.brightness_buttons.replace(buttons);

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
            ("Pulse", "Rapid pulse"),
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
        let color_group = adw::PreferencesGroup::builder().title("Color").build();

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

    fn load_data(&self) {
        let imp = self.imp();

        // Get current brightness via D-Bus and update buttons
        match backend::get_keyboard_brightness_dbus() {
            Ok(current_brightness) => {
                let buttons = imp.brightness_buttons.borrow();
                let index = match current_brightness {
                    KeyboardBrightness::Off => 0,
                    KeyboardBrightness::Low => 1,
                    KeyboardBrightness::Med => 2,
                    KeyboardBrightness::High => 3,
                };

                if let Some(btn) = buttons.get(index) {
                    btn.set_active(true);
                }
            }
            Err(e) => {
                eprintln!("Failed to get keyboard brightness: {}", e);
            }
        }
    }
}

impl Default for AuraPage {
    fn default() -> Self {
        Self::new()
    }
}
