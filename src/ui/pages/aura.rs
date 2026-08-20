use adw::prelude::*;
use gtk4::glib;
use gtk4::glib::prelude::ObjectExt;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::{Cell, RefCell};

use crate::backend::{self, AuraDirection, AuraMode, AuraSpeed, KeyboardBrightness};
use crate::ui::{Refreshable, show_backend_error};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AuraPage {
        pub available: RefCell<bool>,
        pub brightness_buttons: RefCell<Vec<gtk4::ToggleButton>>,
        pub mode_combo: RefCell<Option<adw::ComboRow>>,
        pub supported_modes: RefCell<Vec<AuraMode>>,
        pub speed_buttons: RefCell<Vec<(gtk4::ToggleButton, AuraSpeed)>>,
        pub direction_buttons: RefCell<Vec<(gtk4::ToggleButton, AuraDirection)>>,
        pub color_button: RefCell<Option<gtk4::ColorDialogButton>>,
        pub color2_button: RefCell<Option<gtk4::ColorDialogButton>>,
        pub speed_row: RefCell<Option<adw::ActionRow>>,
        pub direction_row: RefCell<Option<adw::ActionRow>>,
        pub color_row: RefCell<Option<adw::ActionRow>>,
        pub color2_row: RefCell<Option<adw::ActionRow>>,
        pub zone_dropdown: RefCell<Option<gtk4::DropDown>>,
        pub color_group: RefCell<Option<adw::PreferencesGroup>>,
        pub selected_mode: RefCell<AuraMode>,
        pub confirmed_mode: Cell<AuraMode>,
        pub last_successful_zone: Cell<u32>,
        pub refreshing: RefCell<bool>,
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
            self.obj().refresh_data();
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
        let features = backend::detect_features();

        if !features.has_aura {
            *imp.available.borrow_mut() = false;
            let status = adw::StatusPage::builder()
                .icon_name("keyboard-brightness-symbolic")
                .title("Aura Lighting Unavailable")
                .description(if !features.asusctl_installed {
                    "asusctl is not installed. Install asusctl to control keyboard lighting."
                } else {
                    "Aura keyboard lighting is not supported on this device, or the asusd service is not running."
                })
                .vexpand(true)
                .build();
            self.append(&status);
            return;
        }

        *imp.available.borrow_mut() = true;

        let supported_modes: Vec<AuraMode> = if features.aura_modes.is_empty() {
            vec![AuraMode::Static]
        } else {
            features.aura_modes.clone()
        };
        let initial_mode = supported_modes.first().copied().unwrap_or_default();
        *imp.selected_mode.borrow_mut() = initial_mode;
        imp.confirmed_mode.set(initial_mode);
        let supported_zones: Vec<String> = features.aura_zones.clone();

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

            let level_clone = level;
            let this = self.clone();
            btn.connect_clicked(move |button| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if button.is_active() {
                    if let Err(e) = backend::set_keyboard_brightness(level_clone) {
                        log::error!("Failed to set brightness: {e}");
                        show_backend_error(&this, "Couldn’t change keyboard brightness", &e);
                        this.reconcile_brightness_after_write_failure();
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

        // Pre-fetch help texts for all supported modes
        let mode_help: std::collections::HashMap<AuraMode, String> = supported_modes
            .iter()
            .filter_map(|&m| backend::get_aura_mode_help(m).map(|h| (m, h)))
            .collect();
        let mode_help = std::rc::Rc::new(mode_help);

        let mode_labels: Vec<&str> = supported_modes.iter().map(|m| m.label()).collect();
        let mode_list = gtk4::StringList::new(&mode_labels);
        let mode_combo = adw::ComboRow::builder()
            .title("Mode")
            .model(&mode_list)
            .build();

        // Info button with popover showing help for current mode
        let info_popover_label = gtk4::Label::builder()
            .wrap(true)
            .max_width_chars(50)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();
        let info_popover = gtk4::Popover::builder().child(&info_popover_label).build();
        let info_btn = gtk4::MenuButton::builder()
            .icon_name("info-outline-symbolic")
            .popover(&info_popover)
            .valign(gtk4::Align::Center)
            .build();
        info_btn.add_css_class("flat");
        info_btn.add_css_class("circular");

        // Set initial popover text for default mode
        if let Some(first_mode) = supported_modes.first() {
            if let Some(help) = mode_help.get(first_mode) {
                info_popover_label.set_text(help);
            }
        }

        mode_group.set_header_suffix(Some(&info_btn));

        {
            let this = self.clone();
            let modes = supported_modes.clone();
            let help_map = mode_help.clone();
            mode_combo.connect_selected_notify(move |combo| {
                let idx = combo.selected() as usize;
                if let Some(&mode) = modes.get(idx) {
                    *this.imp().selected_mode.borrow_mut() = mode;
                    if let Some(help) = help_map.get(&mode) {
                        info_popover_label.set_text(help);
                    }
                    this.update_mode_visibility();
                    if !*this.imp().refreshing.borrow() {
                        this.apply_aura();
                    }
                }
            });
        }

        mode_group.add(&mode_combo);

        // Speed row
        let speed_row = adw::ActionRow::builder().title("Speed").build();
        let speed_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["linked"])
            .valign(gtk4::Align::Center)
            .build();

        let speeds = [
            (AuraSpeed::Low, "Low"),
            (AuraSpeed::Med, "Med"),
            (AuraSpeed::High, "High"),
        ];

        let mut speed_btns: Vec<(gtk4::ToggleButton, AuraSpeed)> = Vec::new();
        for (speed, label) in speeds {
            let btn = gtk4::ToggleButton::builder().label(label).build();
            let this = self.clone();
            btn.connect_clicked(move |button| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if button.is_active() {
                    this.apply_aura();
                }
            });
            speed_box.append(&btn);
            speed_btns.push((btn, speed));
        }

        for i in 1..speed_btns.len() {
            speed_btns[i].0.set_group(Some(&speed_btns[0].0));
        }
        // Default to Med
        speed_btns[1].0.set_active(true);

        speed_row.add_suffix(&speed_box);
        mode_group.add(&speed_row);

        // Direction row
        let direction_row = adw::ActionRow::builder().title("Direction").build();
        let direction_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["linked"])
            .valign(gtk4::Align::Center)
            .build();

        let directions = [
            (AuraDirection::Up, "Up"),
            (AuraDirection::Down, "Down"),
            (AuraDirection::Left, "Left"),
            (AuraDirection::Right, "Right"),
        ];

        let mut direction_btns: Vec<(gtk4::ToggleButton, AuraDirection)> = Vec::new();
        for (dir, label) in directions {
            let btn = gtk4::ToggleButton::builder().label(label).build();
            let this = self.clone();
            btn.connect_clicked(move |button| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if button.is_active() {
                    this.apply_aura();
                }
            });
            direction_box.append(&btn);
            direction_btns.push((btn, dir));
        }

        for i in 1..direction_btns.len() {
            direction_btns[i].0.set_group(Some(&direction_btns[0].0));
        }
        // Default to Right
        direction_btns[3].0.set_active(true);

        direction_row.add_suffix(&direction_box);
        mode_group.add(&direction_row);

        // Zone row (only shown if device reports supported zones)
        let zone_row = adw::ActionRow::builder().title("Zone").build();

        let mut zone_labels: Vec<&str> = vec!["Default"];
        let zone_strings: Vec<String> = supported_zones;
        for z in &zone_strings {
            zone_labels.push(z.as_str());
        }
        let zone_list = gtk4::StringList::new(&zone_labels);
        let zone_dropdown = gtk4::DropDown::builder()
            .model(&zone_list)
            .valign(gtk4::Align::Center)
            .build();

        let this = self.clone();
        zone_dropdown.connect_selected_notify(move |_| {
            if *this.imp().refreshing.borrow() {
                return;
            }
            this.apply_aura();
        });

        zone_row.add_suffix(&zone_dropdown);
        mode_group.add(&zone_row);

        // Hide zone row if no zones are supported (only "Default" in the list)
        if zone_strings.is_empty() {
            zone_row.set_visible(false);
        }

        self.append(&mode_group);

        // Color selection group
        let color_group = adw::PreferencesGroup::builder().title("Color").build();

        let color_row = adw::ActionRow::builder()
            .title("Primary Color")
            .subtitle("Select keyboard color")
            .build();

        let color_dialog = gtk4::ColorDialog::builder().build();
        let color_button = gtk4::ColorDialogButton::builder()
            .dialog(&color_dialog)
            .valign(gtk4::Align::Center)
            .build();

        let this = self.clone();
        color_button.connect_rgba_notify(move |_| {
            if *this.imp().refreshing.borrow() {
                return;
            }
            this.apply_aura();
        });

        color_row.add_suffix(&color_button);
        color_row.set_activatable_widget(Some(&color_button));
        color_group.add(&color_row);

        // Second color row (for Breathe mode)
        let color2_row = adw::ActionRow::builder()
            .title("Second Color")
            .subtitle("Secondary effect color")
            .build();

        let color2_dialog = gtk4::ColorDialog::builder().build();
        let color2_button = gtk4::ColorDialogButton::builder()
            .dialog(&color2_dialog)
            .valign(gtk4::Align::Center)
            .build();

        let this = self.clone();
        color2_button.connect_rgba_notify(move |_| {
            if *this.imp().refreshing.borrow() {
                return;
            }
            this.apply_aura();
        });

        color2_row.add_suffix(&color2_button);
        color2_row.set_activatable_widget(Some(&color2_button));
        color_group.add(&color2_row);

        self.append(&color_group);

        // Store references
        imp.mode_combo.replace(Some(mode_combo));
        imp.supported_modes.replace(supported_modes);
        imp.speed_buttons.replace(speed_btns);
        imp.direction_buttons.replace(direction_btns);
        imp.color_button.replace(Some(color_button));
        imp.color2_button.replace(Some(color2_button));
        imp.speed_row.replace(Some(speed_row));
        imp.direction_row.replace(Some(direction_row));
        imp.zone_dropdown.replace(Some(zone_dropdown));
        imp.color_row.replace(Some(color_row));
        imp.color2_row.replace(Some(color2_row));
        imp.color_group.replace(Some(color_group));

        // Set default mode and visibility
        self.update_mode_visibility();
    }

    /// Show/hide rows based on the selected mode
    fn update_mode_visibility(&self) {
        let imp = self.imp();
        let mode = *imp.selected_mode.borrow();

        if let Some(row) = imp.speed_row.borrow().as_ref() {
            row.set_visible(mode.needs_speed());
        }
        if let Some(row) = imp.direction_row.borrow().as_ref() {
            row.set_visible(mode.needs_direction());
        }
        let needs_any_colour = mode.needs_colour() || mode.needs_colour2();
        if let Some(group) = imp.color_group.borrow().as_ref() {
            group.set_visible(needs_any_colour);
        }
        if let Some(row) = imp.color_row.borrow().as_ref() {
            row.set_visible(mode.needs_colour());
        }
        if let Some(row) = imp.color2_row.borrow().as_ref() {
            row.set_visible(mode.needs_colour2());
        }
    }

    /// Apply the currently selected aura settings via CLI
    fn apply_aura(&self) {
        let imp = self.imp();
        let mode = *imp.selected_mode.borrow();

        let colour = imp
            .color_button
            .borrow()
            .as_ref()
            .map(|b| rgba_to_hex(&b.rgba()));
        let colour2 = imp
            .color2_button
            .borrow()
            .as_ref()
            .map(|b| rgba_to_hex(&b.rgba()));

        let speed = imp
            .speed_buttons
            .borrow()
            .iter()
            .find(|(btn, _)| btn.is_active())
            .map(|(_, s)| *s);

        let direction = imp
            .direction_buttons
            .borrow()
            .iter()
            .find(|(btn, _)| btn.is_active())
            .map(|(_, d)| *d);

        let (zone_index, zone) = if let Some(dropdown) = imp.zone_dropdown.borrow().as_ref() {
            let selected = dropdown.selected();
            let zone = if selected == 0 {
                None
            } else {
                dropdown
                    .selected_item()
                    .and_downcast::<gtk4::StringObject>()
                    .map(|obj| obj.string().to_string())
            };
            (selected, zone)
        } else {
            (0, None)
        };

        match backend::set_aura_mode(
            mode,
            colour.as_deref(),
            colour2.as_deref(),
            speed,
            direction,
            zone.as_deref(),
        ) {
            Ok(()) => {
                imp.confirmed_mode.set(mode);
                imp.last_successful_zone.set(zone_index);
            }
            Err(e) => {
                log::error!("Failed to set aura mode: {e}");
                show_backend_error(self, "Couldn’t change Aura lighting", &e);
                self.reconcile_aura_after_write_failure();
            }
        }
    }

    fn apply_keyboard_brightness(&self, brightness: KeyboardBrightness) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        {
            let buttons = imp.brightness_buttons.borrow();
            let index = match brightness {
                KeyboardBrightness::Off => 0,
                KeyboardBrightness::Low => 1,
                KeyboardBrightness::Med => 2,
                KeyboardBrightness::High => 3,
            };

            let _guards: Vec<_> = buttons
                .iter()
                .map(|button| button.freeze_notify())
                .collect();
            if let Some(button) = buttons.get(index) {
                button.set_active(true);
            }
        }

        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_brightness_after_write_failure(&self) {
        match backend::get_keyboard_brightness_dbus() {
            Ok(brightness) => self.apply_keyboard_brightness(brightness),
            Err(e) => {
                log::error!("Failed to reconcile keyboard brightness after write failure: {e}");
            }
        }
    }

    fn apply_aura_mode_state(
        &self,
        mode: AuraMode,
        speed: AuraSpeed,
        direction: AuraDirection,
        color1: (u8, u8, u8),
        color2: (u8, u8, u8),
        zone_index: Option<u32>,
    ) {
        let imp = self.imp();
        *imp.refreshing.borrow_mut() = true;

        {
            let modes = imp.supported_modes.borrow();
            if let Some(index) = modes.iter().position(|supported| *supported == mode) {
                if let Some(combo) = imp.mode_combo.borrow().as_ref() {
                    let _guard = combo.freeze_notify();
                    combo.set_selected(index as u32);
                }
                *imp.selected_mode.borrow_mut() = mode;
                imp.confirmed_mode.set(mode);
            }
        }

        {
            let speed_buttons = imp.speed_buttons.borrow();
            let _guards: Vec<_> = speed_buttons
                .iter()
                .map(|(button, _)| button.freeze_notify())
                .collect();
            for (button, button_speed) in speed_buttons.iter() {
                if *button_speed == speed {
                    button.set_active(true);
                    break;
                }
            }
        }

        {
            let direction_buttons = imp.direction_buttons.borrow();
            let _guards: Vec<_> = direction_buttons
                .iter()
                .map(|(button, _)| button.freeze_notify())
                .collect();
            for (button, button_direction) in direction_buttons.iter() {
                if *button_direction == direction {
                    button.set_active(true);
                    break;
                }
            }
        }

        if let Some(color_button) = imp.color_button.borrow().as_ref() {
            let _guard = color_button.freeze_notify();
            let (red, green, blue) = color1;
            color_button.set_rgba(&gtk4::gdk::RGBA::new(
                red as f32 / 255.0,
                green as f32 / 255.0,
                blue as f32 / 255.0,
                1.0,
            ));
        }

        if let Some(color_button) = imp.color2_button.borrow().as_ref() {
            let _guard = color_button.freeze_notify();
            let (red, green, blue) = color2;
            color_button.set_rgba(&gtk4::gdk::RGBA::new(
                red as f32 / 255.0,
                green as f32 / 255.0,
                blue as f32 / 255.0,
                1.0,
            ));
        }

        if let (Some(index), Some(dropdown)) = (zone_index, imp.zone_dropdown.borrow().as_ref()) {
            let _guard = dropdown.freeze_notify();
            dropdown.set_selected(index);
        }

        self.update_mode_visibility();
        *imp.refreshing.borrow_mut() = false;
    }

    fn reconcile_aura_after_write_failure(&self) {
        match backend::get_aura_mode_data_dbus() {
            Ok(mode_data) => self.apply_aura_mode_state(
                mode_data.mode,
                mode_data.speed,
                mode_data.direction,
                mode_data.color1,
                mode_data.color2,
                Some(self.imp().last_successful_zone.get()),
            ),
            Err(e) => {
                log::error!("Failed to reconcile Aura lighting after write failure: {e}");
                self.restore_confirmed_mode();
            }
        }
    }

    fn restore_confirmed_mode(&self) {
        let imp = self.imp();
        let mode = imp.confirmed_mode.get();
        *imp.refreshing.borrow_mut() = true;

        {
            let modes = imp.supported_modes.borrow();
            if let Some(index) = modes.iter().position(|supported| *supported == mode) {
                if let Some(combo) = imp.mode_combo.borrow().as_ref() {
                    let _guard = combo.freeze_notify();
                    combo.set_selected(index as u32);
                }
                *imp.selected_mode.borrow_mut() = mode;
            }
        }

        self.update_mode_visibility();
        *imp.refreshing.borrow_mut() = false;
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        let imp = self.imp();

        if !*imp.available.borrow() {
            return;
        }

        // Get current brightness via D-Bus and update buttons
        match backend::get_keyboard_brightness_dbus() {
            Ok(current_brightness) => self.apply_keyboard_brightness(current_brightness),
            Err(e) => {
                log::error!("Failed to get keyboard brightness: {e}");
            }
        }

        // Get full aura mode data via D-Bus and update all controls
        match backend::get_aura_mode_data_dbus() {
            Ok(mode_data) => self.apply_aura_mode_state(
                mode_data.mode,
                mode_data.speed,
                mode_data.direction,
                mode_data.color1,
                mode_data.color2,
                None,
            ),
            Err(e) => {
                log::error!("Failed to get aura mode data: {e}");
            }
        }
    }
}

fn rgba_to_hex(rgba: &gtk4::gdk::RGBA) -> String {
    let r = (rgba.red() * 255.0) as u8;
    let g = (rgba.green() * 255.0) as u8;
    let b = (rgba.blue() * 255.0) as u8;
    format!("{r:02x}{g:02x}{b:02x}")
}

impl Default for AuraPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Refreshable for AuraPage {
    fn refresh(&self) {
        self.refresh_data();
    }
}
