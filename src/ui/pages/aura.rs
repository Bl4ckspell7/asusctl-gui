use adw::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;

use crate::backend::{self, AuraDirection, AuraMode, AuraSpeed, KeyboardBrightness};
use crate::ui::Refreshable;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct AuraPage {
        pub brightness_buttons: RefCell<Vec<gtk4::ToggleButton>>,
        pub mode_buttons: RefCell<Vec<(gtk4::ToggleButton, AuraMode)>>,
        pub speed_buttons: RefCell<Vec<(gtk4::ToggleButton, AuraSpeed)>>,
        pub direction_buttons: RefCell<Vec<(gtk4::ToggleButton, AuraDirection)>>,
        pub color_button: RefCell<Option<gtk4::ColorDialogButton>>,
        pub color2_button: RefCell<Option<gtk4::ColorDialogButton>>,
        pub speed_row: RefCell<Option<adw::ActionRow>>,
        pub direction_row: RefCell<Option<adw::ActionRow>>,
        pub color_row: RefCell<Option<adw::ActionRow>>,
        pub color2_row: RefCell<Option<adw::ActionRow>>,
        pub zone_dropdown: RefCell<Option<gtk4::DropDown>>,
        pub selected_mode: RefCell<AuraMode>,
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

        // Query supported features from asusctl
        let supported = backend::get_supported_features().ok();
        let supported_modes: Vec<AuraMode> = supported
            .as_ref()
            .map(|s| s.aura_modes.clone())
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| AuraMode::ALL.to_vec());
        let supported_zones: Vec<String> = supported
            .as_ref()
            .map(|s| s.aura_zones.clone())
            .unwrap_or_default();

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
                        eprintln!("Failed to set brightness: {e}");
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

        let mode_row = adw::ActionRow::builder().title("Mode").build();
        let mode_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["linked"])
            .valign(gtk4::Align::Center)
            .build();

        // Pre-fetch help texts for all supported modes
        let mode_help: std::collections::HashMap<AuraMode, String> = supported_modes
            .iter()
            .filter_map(|&m| backend::get_aura_mode_help(m).map(|h| (m, h)))
            .collect();

        // Info button with popover showing help for current mode
        let info_popover_label = gtk4::Label::builder()
            .wrap(true)
            .max_width_chars(50)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();
        let info_popover = gtk4::Popover::builder()
            .child(&info_popover_label)
            .build();
        let info_btn = gtk4::MenuButton::builder()
            .icon_name("info-outline-symbolic")
            .popover(&info_popover)
            .valign(gtk4::Align::Center)
            .build();
        info_btn.add_css_class("flat");
        info_btn.add_css_class("circular");

        let mode_help = std::rc::Rc::new(mode_help);

        let mut mode_btns: Vec<(gtk4::ToggleButton, AuraMode)> = Vec::new();
        for &mode in &supported_modes {
            let btn = gtk4::ToggleButton::builder().label(mode.label()).build();

            let this = self.clone();
            let popover_label = info_popover_label.clone();
            let help_map = mode_help.clone();
            btn.connect_clicked(move |button| {
                if *this.imp().refreshing.borrow() {
                    return;
                }
                if button.is_active() {
                    *this.imp().selected_mode.borrow_mut() = mode;
                    // Update info popover text for current mode
                    if let Some(help) = help_map.get(&mode) {
                        popover_label.set_text(help);
                    }
                    this.update_mode_visibility();
                    this.apply_aura();
                }
            });
            mode_box.append(&btn);
            mode_btns.push((btn, mode));
        }

        // Set initial popover text for default mode
        if let Some(first_mode) = supported_modes.first() {
            if let Some(help) = mode_help.get(first_mode) {
                info_popover_label.set_text(help);
            }
        }

        // Link mode buttons together
        for i in 1..mode_btns.len() {
            mode_btns[i].0.set_group(Some(&mode_btns[0].0));
        }

        let mode_suffix = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();
        mode_suffix.append(&mode_box);
        mode_suffix.append(&info_btn);
        mode_row.add_suffix(&mode_suffix);
        mode_group.add(&mode_row);

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
        let zone_row = adw::ActionRow::builder()
            .title("Zone")
            .build();

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
            .subtitle("Breathing target color")
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
        imp.mode_buttons.replace(mode_btns);
        imp.speed_buttons.replace(speed_btns);
        imp.direction_buttons.replace(direction_btns);
        imp.color_button.replace(Some(color_button));
        imp.color2_button.replace(Some(color2_button));
        imp.speed_row.replace(Some(speed_row));
        imp.direction_row.replace(Some(direction_row));
        imp.zone_dropdown.replace(Some(zone_dropdown));
        imp.color_row.replace(Some(color_row));
        imp.color2_row.replace(Some(color2_row));

        // Set default mode and visibility
        mode_btns_first_active(&imp.mode_buttons.borrow());
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

        let zone = imp.zone_dropdown.borrow().as_ref().and_then(|dd| {
            let selected = dd.selected();
            if selected == 0 {
                None // "Default" = no zone arg
            } else {
                dd.selected_item()
                    .and_downcast::<gtk4::StringObject>()
                    .map(|obj| obj.string().to_string())
            }
        });

        if let Err(e) = backend::set_aura_mode(
            mode,
            colour.as_deref(),
            colour2.as_deref(),
            speed,
            direction,
            zone.as_deref(),
        ) {
            eprintln!("Failed to set aura mode: {e}");
        }
    }

    /// Refresh/reload all data on this page
    fn refresh_data(&self) {
        let imp = self.imp();

        *imp.refreshing.borrow_mut() = true;

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
                eprintln!("Failed to get keyboard brightness: {e}");
            }
        }

        *imp.refreshing.borrow_mut() = false;
    }
}

fn rgba_to_hex(rgba: &gtk4::gdk::RGBA) -> String {
    let r = (rgba.red() * 255.0) as u8;
    let g = (rgba.green() * 255.0) as u8;
    let b = (rgba.blue() * 255.0) as u8;
    format!("{r:02x}{g:02x}{b:02x}")
}

fn mode_btns_first_active(btns: &[(gtk4::ToggleButton, AuraMode)]) {
    if let Some((btn, _)) = btns.first() {
        btn.set_active(true);
    }
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
