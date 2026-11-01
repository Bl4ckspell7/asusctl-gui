use adw::prelude::*;
use gtk4::glib;
use gtk4::glib::prelude::ObjectExt;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, GpuMode, GpuState};
use crate::ui::{Refreshable, show_backend_error};

const REBOOT_RESPONSE: &str = "reboot";
const CANCEL_RESPONSE: &str = "cancel";

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct GpuPage {
        pub available: RefCell<bool>,
        pub modes: RefCell<Vec<GpuMode>>,
        pub mode_rows: RefCell<Vec<adw::ActionRow>>,
        pub mode_radios: RefCell<Vec<gtk4::CheckButton>>,
        pub active_labels: RefCell<Vec<gtk4::Label>>,
        pub mode_group: RefCell<Option<adw::PreferencesGroup>>,
        pub reboot_banner: RefCell<Option<adw::Banner>>,
        pub service_warning: RefCell<Option<adw::PreferencesGroup>>,
        /// Flag to prevent signal handlers from firing during refresh
        pub refreshing: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GpuPage {
        const NAME: &'static str = "GpuPage";
        type Type = super::GpuPage;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for GpuPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
            self.obj().refresh_data();
        }
    }

    impl WidgetImpl for GpuPage {}
    impl BoxImpl for GpuPage {}
}

glib::wrapper! {
    pub struct GpuPage(ObjectSubclass<imp::GpuPage>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl GpuPage {
    pub fn new() -> Self {
        // No margins on the page itself so the reboot banner spans the full width;
        // the content box below carries the usual page margins.
        glib::Object::builder()
            .property("orientation", gtk4::Orientation::Vertical)
            .build()
    }

    fn setup_ui(&self) {
        let imp = self.imp();
        let features = backend::detect_features();
        let modes = backend::available_gpu_modes(features);

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(24)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .vexpand(true)
            .build();

        if !features.asusctl_installed || modes.is_empty() {
            *imp.available.borrow_mut() = false;
            let status = adw::StatusPage::builder()
                .icon_name("video-display-symbolic")
                .title("GPU Switching Unavailable")
                .description(if !features.asusctl_installed {
                    "asusctl is not installed. Install asusctl to switch GPU modes."
                } else {
                    "GPU switching is not supported on this device, the asus-armoury driver is not loaded, or the asusd service is not running."
                })
                .vexpand(true)
                .build();
            content.append(&status);
            self.append(&content);
            return;
        }

        *imp.available.borrow_mut() = true;

        // Pending change banner
        let reboot_banner = adw::Banner::builder()
            .button_label("Reboot Now")
            .button_style(adw::BannerButtonStyle::Suggested)
            .revealed(false)
            .build();

        {
            let this = self.clone();
            reboot_banner.connect_button_clicked(move |_| {
                this.request_reboot();
            });
        }

        imp.reboot_banner.replace(Some(reboot_banner.clone()));
        self.append(&reboot_banner);

        // Page title
        let title = gtk4::Label::builder()
            .label("GPU")
            .css_classes(["title-1"])
            .halign(gtk4::Align::Start)
            .build();

        content.append(&title);

        // Description
        let description = gtk4::Label::builder()
            .label("Choose which graphics card drives the display")
            .css_classes(["dim-label"])
            .halign(gtk4::Align::Start)
            .build();

        content.append(&description);

        // Warning when queued changes can't be applied on shutdown
        let service_warning = adw::PreferencesGroup::builder().visible(false).build();

        let service_warning_row = adw::ActionRow::builder()
            .title("GPU changes can’t be applied")
            .subtitle("asus-shutdown.service is not running, so queued changes would not be applied on reboot")
            .build();

        let warning_icon = gtk4::Image::from_icon_name("dialog-warning-symbolic");
        warning_icon.add_css_class("warning");
        service_warning_row.add_prefix(&warning_icon);
        service_warning.add(&service_warning_row);

        imp.service_warning.replace(Some(service_warning.clone()));
        content.append(&service_warning);

        // GPU mode group
        let mode_group = adw::PreferencesGroup::builder()
            .title("GPU Mode")
            .description("Changes are applied while the system shuts down and take effect after a reboot. Logging out is not enough.")
            .build();

        let mut rows: Vec<adw::ActionRow> = Vec::new();
        let mut radios: Vec<gtk4::CheckButton> = Vec::new();
        let mut active_labels: Vec<gtk4::Label> = Vec::new();
        let mut first_radio: Option<gtk4::CheckButton> = None;

        for mode in &modes {
            let row = adw::ActionRow::builder()
                .title(mode.label())
                .subtitle(mode.description())
                .activatable(true)
                .build();

            let active_label = gtk4::Label::builder()
                .label("Active")
                .css_classes(["dim-label"])
                .valign(gtk4::Align::Center)
                .visible(false)
                .build();

            let radio = gtk4::CheckButton::builder()
                .valign(gtk4::Align::Center)
                .build();

            // Set the group for radio button behavior
            if let Some(ref group) = first_radio {
                radio.set_group(Some(group));
            } else {
                first_radio = Some(radio.clone());
            }

            // Connect toggled handler to queue the GPU mode
            let mode = *mode;
            let this = self.clone();
            radio.connect_toggled(move |button| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if button.is_active() {
                    if let Err(e) = backend::set_gpu_mode(mode) {
                        log::error!("Failed to set GPU mode: {e}");
                        show_backend_error(&this, "Couldn’t change the GPU mode", &e);
                    }
                    this.refresh_data();
                }
            });

            row.add_suffix(&active_label);
            row.add_suffix(&radio);
            row.set_activatable_widget(Some(&radio));

            radios.push(radio);
            active_labels.push(active_label);
            mode_group.add(&row);
            rows.push(row);
        }

        imp.modes.replace(modes);
        imp.mode_rows.replace(rows);
        imp.mode_radios.replace(radios);
        imp.active_labels.replace(active_labels);
        imp.mode_group.replace(Some(mode_group.clone()));
        content.append(&mode_group);

        self.append(&content);
    }

    fn apply_gpu_state(&self, state: GpuState, shutdown_service_active: bool) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        let modes = imp.modes.borrow();
        let selected = state.pending.unwrap_or(state.active);

        {
            let radios = imp.mode_radios.borrow();
            let _guards: Vec<_> = radios.iter().map(|radio| radio.freeze_notify()).collect();
            if let Some(radio) = modes
                .iter()
                .position(|mode| *mode == selected)
                .and_then(|index| radios.get(index))
            {
                radio.set_active(true);
            }
        }

        for (mode, label) in modes.iter().zip(imp.active_labels.borrow().iter()) {
            label.set_visible(*mode == state.active);
        }

        for (mode, row) in modes.iter().zip(imp.mode_rows.borrow().iter()) {
            let blocked = mode.blocked_from(state.active);
            row.set_sensitive(blocked.is_none());
            row.set_subtitle(blocked.unwrap_or(mode.description()));
        }

        if let Some(banner) = imp.reboot_banner.borrow().as_ref() {
            match state.pending {
                Some(pending) => {
                    banner.set_title(&format!("Switching to {pending} mode after reboot"));
                    banner.set_revealed(true);
                }
                None => banner.set_revealed(false),
            }
        }

        if let Some(warning) = imp.service_warning.borrow().as_ref() {
            warning.set_visible(!shutdown_service_active);
        }

        // Re-enable after a failed read
        if let Some(group) = imp.mode_group.borrow().as_ref() {
            group.set_sensitive(true);
        }

        *imp.refreshing.borrow_mut() = false;
    }

    /// Disable the mode choices while the GPU state can't be read, so no write is
    /// queued blindly.
    fn apply_unknown_state(&self) {
        let imp = self.imp();

        if let Some(banner) = imp.reboot_banner.borrow().as_ref() {
            banner.set_revealed(false);
        }

        if let Some(group) = imp.mode_group.borrow().as_ref() {
            group.set_sensitive(false);
        }
    }

    /// Reboot through GNOME's own confirmation dialog, or confirm here first
    /// on other desktops.
    fn request_reboot(&self) {
        match backend::request_session_reboot() {
            Ok(true) => {}
            Ok(false) => self.confirm_reboot(),
            Err(e) => {
                log::error!("Failed to request reboot: {e}");
                show_backend_error(self, "Couldn’t start the reboot", &e);
            }
        }
    }

    fn confirm_reboot(&self) {
        let dialog = adw::AlertDialog::new(
            Some("Reboot Now?"),
            Some(
                "Save your work first. The GPU mode change is applied while the system shuts down.",
            ),
        );
        dialog.add_responses(&[(CANCEL_RESPONSE, "Cancel"), (REBOOT_RESPONSE, "Reboot")]);
        dialog.set_response_appearance(REBOOT_RESPONSE, adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some(CANCEL_RESPONSE));
        dialog.set_close_response(CANCEL_RESPONSE);

        let this = self.clone();
        dialog.connect_response(None, move |_, response| {
            if response != REBOOT_RESPONSE {
                return;
            }
            if let Err(e) = backend::reboot_now() {
                log::error!("Failed to reboot: {e}");
                show_backend_error(&this, "Couldn’t start the reboot", &e);
            }
        });

        dialog.present(Some(self));
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        if !*self.imp().available.borrow() {
            return;
        }

        match backend::get_gpu_state() {
            Ok(state) => {
                // The service only matters while a change is waiting for it
                let shutdown_service_active =
                    state.pending.is_none() || backend::is_shutdown_service_active();
                self.apply_gpu_state(state, shutdown_service_active);
            }
            Err(e) => {
                log::error!("Failed to get GPU state: {e}");
                self.apply_unknown_state();
            }
        }
    }
}

impl Default for GpuPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Refreshable for GpuPage {
    fn refresh(&self) {
        self.refresh_data();
    }
}
