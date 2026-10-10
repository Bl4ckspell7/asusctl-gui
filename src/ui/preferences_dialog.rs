use adw::prelude::*;
use gtk4::gio;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::Cell;
use std::rc::Rc;

use super::Page;
use crate::backend;

const SHORTCUT_OFF_SUBTITLE: &str = "Adds a custom shortcut in GNOME Settings";
const SHORTCUT_NO_KEY_SUBTITLE: &str = "No key assigned, set one in GNOME Settings";
const SHORTCUT_ERROR: &str = "Couldn’t change the keyboard shortcut";

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

        // Create refresh interval spin row (0.1-1.0 seconds)
        let refresh_interval_row = adw::SpinRow::builder()
            .title("Refresh Interval")
            .subtitle("How often the visible page reloads system data, in seconds")
            .adjustment(&gtk4::Adjustment::new(
                0.5, // default value
                0.1, // min
                1.0, // max
                0.1, // step increment
                0.1, // page increment
                0.0, // page size
            ))
            .digits(1)
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

        self.setup_shortcut_group(&general_page);

        self.add(&general_page);
    }

    /// Add the "Open with ROG key" switch, backed by a GNOME custom shortcut.
    ///
    /// The group stays hidden until the current shortcut has been read, and
    /// for good if it cannot be read (not GNOME, or no host `gsettings`).
    fn setup_shortcut_group(&self, page: &adw::PreferencesPage) {
        if !backend::is_gnome() {
            return;
        }

        let shortcut_group = adw::PreferencesGroup::builder()
            .title("Keyboard Shortcut")
            .description("Open ASUS Control from anywhere using a GNOME custom shortcut")
            .visible(false)
            .build();

        let shortcut_row = adw::SwitchRow::builder().title("Open with ROG key").build();

        shortcut_group.add(&shortcut_row);
        page.add(&shortcut_group);

        // Reading runs one host gsettings call per custom shortcut, so it must
        // not run on the main loop.
        let dialog_weak = self.downgrade();
        let group_weak = shortcut_group.downgrade();
        let row_weak = shortcut_row.downgrade();
        glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(backend::get_launch_shortcut).await;

            let (Some(dialog), Some(group), Some(row)) = (
                dialog_weak.upgrade(),
                group_weak.upgrade(),
                row_weak.upgrade(),
            ) else {
                return;
            };

            match result {
                Ok(Ok(shortcut)) => {
                    // Connect only after the initial state is set, so loading
                    // never writes the shortcut
                    Self::apply_shortcut_state(&row, shortcut.as_deref());
                    dialog.connect_shortcut_row(&row);
                    group.set_visible(true);
                }
                Ok(Err(e)) => log::warn!("Keyboard shortcut unavailable: {e}"),
                Err(_) => log::warn!("Keyboard shortcut unavailable: the reading thread panicked"),
            }
        });
    }

    fn connect_shortcut_row(&self, row: &adw::SwitchRow) {
        // Set while the switch is updated programmatically, so reverting it
        // after a failure does not write the shortcut again
        let syncing = Rc::new(Cell::new(false));

        let dialog_weak = self.downgrade();
        row.connect_active_notify(move |row| {
            if syncing.get() {
                return;
            }

            let enable = row.is_active();
            row.set_sensitive(false);

            let dialog_weak = dialog_weak.clone();
            let row_weak = row.downgrade();
            let syncing = syncing.clone();
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || {
                    let change = if enable {
                        backend::add_launch_shortcut()
                    } else {
                        backend::remove_launch_shortcut()
                    };
                    // Re-read even after a failure, the change may have partly applied
                    (change, backend::get_launch_shortcut())
                })
                .await;

                let Some(row) = row_weak.upgrade() else {
                    return;
                };
                row.set_sensitive(true);

                syncing.set(true);
                let error = match result {
                    Ok((change, Ok(binding))) => {
                        Self::apply_shortcut_state(&row, binding.as_deref());
                        change.err().map(|e| e.to_string())
                    }
                    Ok((_, Err(e))) => {
                        row.set_active(!enable);
                        Some(e.to_string())
                    }
                    Err(_) => {
                        row.set_active(!enable);
                        Some("the shortcut thread panicked".to_string())
                    }
                };
                syncing.set(false);

                if let Some(error) = error {
                    log::warn!("{SHORTCUT_ERROR}: {error}");
                    if let Some(dialog) = dialog_weak.upgrade() {
                        dialog.add_toast(adw::Toast::new(SHORTCUT_ERROR));
                    }
                }
            });
        });
    }

    fn apply_shortcut_state(row: &adw::SwitchRow, binding: Option<&str>) {
        row.set_active(binding.is_some());

        let subtitle = match binding {
            None => SHORTCUT_OFF_SUBTITLE.to_string(),
            Some("") => SHORTCUT_NO_KEY_SUBTITLE.to_string(),
            Some(binding) => format!("Shortcut: {}", Self::shortcut_label(binding)),
        };
        row.set_subtitle(&subtitle);
    }

    /// Human-readable label for a GNOME accelerator, e.g. `<Control><Alt>a` → `Ctrl+Alt+A`.
    fn shortcut_label(binding: &str) -> String {
        match gtk4::accelerator_parse(binding) {
            Some((key, mods)) => gtk4::accelerator_get_label(key, mods).to_string(),
            None => binding.to_string(),
        }
    }
}
