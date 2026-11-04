use adw::prelude::*;
use gtk4::glib;
use gtk4::glib::prelude::ObjectExt;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend;
use crate::ui::{Refreshable, show_backend_error};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct SystemPage {
        pub available: RefCell<bool>,
        pub boot_sound_row: RefCell<Option<adw::SwitchRow>>,
        /// Flag to prevent signal handlers from firing during refresh
        pub refreshing: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemPage {
        const NAME: &'static str = "SystemPage";
        type Type = super::SystemPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for SystemPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().refresh_data();
        }
    }

    impl WidgetImpl for SystemPage {}
    impl BoxImpl for SystemPage {}
}

glib::wrapper! {
    pub struct SystemPage(ObjectSubclass<imp::SystemPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl SystemPage {
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

        if !features.asusctl_installed || !features.has_boot_sound {
            *imp.available.borrow_mut() = false;
            let status = adw::StatusPage::builder()
                .icon_name("emblem-system-symbolic")
                .title("System Settings Unavailable")
                .description(if !features.asusctl_installed {
                    "asusctl is not installed. Install asusctl to change firmware settings."
                } else {
                    "Firmware settings are not supported on this device, the asus-armoury driver is not loaded, or the asusd service is not running."
                })
                .vexpand(true)
                .build();
            self.append(&status);
            return;
        }

        *imp.available.borrow_mut() = true;

        // Page title
        let title = gtk4::Label::builder()
            .label("System")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&title);

        // Description
        let description = gtk4::Label::builder()
            .label("Settings stored in the laptop’s firmware")
            .css_classes(["dim-label"])
            .halign(gtk4::Align::Start)
            .build();

        self.append(&description);

        // Firmware group
        let firmware_group = adw::PreferencesGroup::builder().title("Firmware").build();

        let boot_sound_row = adw::SwitchRow::builder()
            .title("Boot Sound")
            .subtitle("Play the startup sound when the laptop powers on")
            .build();

        // Connect the switch to turn the boot sound on or off
        {
            let this = self.clone();
            boot_sound_row.connect_active_notify(move |row| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                let enabled = row.is_active();
                if let Err(e) = backend::set_boot_sound(enabled) {
                    log::error!("Failed to set boot sound: {e}");
                    show_backend_error(&this, "Couldn’t change the boot sound", &e);
                    if !this.reconcile_boot_sound_after_write_failure() {
                        // Couldn't read the real value either: undo the toggle
                        this.apply_boot_sound(!enabled);
                    }
                }
            });
        }

        imp.boot_sound_row.replace(Some(boot_sound_row.clone()));
        firmware_group.add(&boot_sound_row);
        self.append(&firmware_group);
    }

    fn apply_boot_sound(&self, enabled: bool) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        if let Some(row) = imp.boot_sound_row.borrow().as_ref() {
            let _guard = row.freeze_notify();
            row.set_active(enabled);
            // Re-enable after a failed read
            row.set_sensitive(true);
        }

        *imp.refreshing.borrow_mut() = false;
    }

    /// Re-read the boot sound after a failed write. Returns whether the read worked.
    fn reconcile_boot_sound_after_write_failure(&self) -> bool {
        match backend::get_boot_sound() {
            Ok(enabled) => {
                self.apply_boot_sound(enabled);
                true
            }
            Err(e) => {
                log::error!("Failed to reconcile boot sound after write failure: {e}");
                false
            }
        }
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        if !*self.imp().available.borrow() {
            return;
        }

        match backend::get_boot_sound() {
            Ok(enabled) => self.apply_boot_sound(enabled),
            Err(e) => {
                log::error!("Failed to get boot sound: {e}");
                // Don't offer a switch whose real value is unknown
                if let Some(row) = self.imp().boot_sound_row.borrow().as_ref() {
                    row.set_sensitive(false);
                }
            }
        }
    }
}

impl Default for SystemPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Refreshable for SystemPage {
    fn refresh(&self) {
        self.refresh_data();
    }
}
