use adw::prelude::*;
use gtk4::gio;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use libadwaita as adw;

use super::Page;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct PreferencesDialog {
        pub settings: RefCell<Option<gio::Settings>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesDialog {
        const NAME: &'static str = "PreferencesDialog";
        type Type = super::PreferencesDialog;
        type ParentType = adw::PreferencesDialog;
    }

    impl ObjectImpl for PreferencesDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let settings = gio::Settings::new("com.github.bl4ckspell7.asusctl-gui");
            self.settings.replace(Some(settings));

            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for PreferencesDialog {}
    impl AdwDialogImpl for PreferencesDialog {}
    impl PreferencesDialogImpl for PreferencesDialog {}
}

glib::wrapper! {
    pub struct PreferencesDialog(ObjectSubclass<imp::PreferencesDialog>)
        @extends adw::PreferencesDialog, adw::Dialog, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn settings(&self) -> gio::Settings {
        self.imp()
            .settings
            .borrow()
            .clone()
            .expect("Settings not initialized")
    }

    fn setup_ui(&self) {
        self.set_title("Preferences");
        self.set_search_enabled(false);

        let settings = self.settings();

        // Create the General preferences page
        let general_page = adw::PreferencesPage::builder()
            .title("General")
            .icon_name("preferences-system-symbolic")
            .build();

        // Create the Startup group
        let startup_group = adw::PreferencesGroup::builder()
            .title("Startup")
            .description("Configure which page opens when the application starts")
            .build();

        // Create the "Open on page" combo row using Page enum
        let page_titles: Vec<&str> = Page::ALL.iter().map(|p| p.title()).collect();
        let page_options = gtk4::StringList::new(&page_titles);

        let startup_page_row = adw::ComboRow::builder()
            .title("Open on page")
            .subtitle("Select which page to show on startup")
            .model(&page_options)
            .build();

        // Create the "Restore last page" switch row
        let restore_last_row = adw::SwitchRow::builder()
            .title("Restore last page")
            .subtitle("Open the page you were on when you last closed the app")
            .build();

        // Set initial state for restore-last-page switch
        let restore_last = settings.boolean("restore-last-page");
        restore_last_row.set_active(restore_last);

        // Set initial state for startup-page combo and sensitivity
        startup_page_row.set_sensitive(!restore_last);
        let startup_page_str = settings.string("startup-page");
        let startup_page = Page::try_from(startup_page_str.as_str()).unwrap_or_default();
        startup_page_row.set_selected(startup_page.index());

        // Connect restore-last-page switch
        let settings_clone = settings.clone();
        let startup_page_row_clone = startup_page_row.clone();
        restore_last_row.connect_active_notify(move |switch| {
            let active = switch.is_active();
            let _ = settings_clone.set_boolean("restore-last-page", active);
            // Disable the page selector when "restore last page" is enabled
            startup_page_row_clone.set_sensitive(!active);
        });

        // Connect startup-page combo
        let settings_clone = settings.clone();
        startup_page_row.connect_selected_notify(move |combo| {
            if let Some(page) = Page::from_index(combo.selected()) {
                let _ = settings_clone.set_string("startup-page", page.as_str());
            }
        });

        startup_group.add(&restore_last_row);
        startup_group.add(&startup_page_row);
        general_page.add(&startup_group);

        // Create the Refresh group
        let refresh_group = adw::PreferencesGroup::builder().title("General").build();

        // Create refresh interval spin row (0.1-10.0 seconds)
        let refresh_interval_row = adw::SpinRow::builder()
            .title("Update Interval")
            .subtitle("In seconds")
            .adjustment(&gtk4::Adjustment::new(
                0.5,  // default value
                0.1,  // min
                10.0, // max
                0.1,  // step increment
                1.0,  // page increment
                0.0,  // page size
            ))
            .digits(2)
            .build();

        // Load current refresh interval
        let current_interval = settings.double("refresh-interval");
        refresh_interval_row.set_value(current_interval);

        // Connect refresh interval change
        let settings_clone = settings;
        refresh_interval_row.connect_value_notify(move |spin_row| {
            let _ = settings_clone.set_double("refresh-interval", spin_row.value());
        });

        refresh_group.add(&refresh_interval_row);
        general_page.add(&refresh_group);

        self.add(&general_page);
    }
}
