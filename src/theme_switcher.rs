use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ThemeSwitcher;

    #[glib::object_subclass]
    impl ObjectSubclass for ThemeSwitcher {
        const NAME: &'static str = "ThemeSwitcher";
        type Type = super::ThemeSwitcher;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for ThemeSwitcher {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_ui();
        }
    }

    impl WidgetImpl for ThemeSwitcher {}
    impl BoxImpl for ThemeSwitcher {}
}

glib::wrapper! {
    pub struct ThemeSwitcher(ObjectSubclass<imp::ThemeSwitcher>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for ThemeSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeSwitcher {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_ui(&self) {
        self.add_css_class("themeswitcher");
        self.set_halign(gtk4::Align::Fill);
        self.set_hexpand(true);
        self.set_homogeneous(true);
        self.set_margin_start(12);
        self.set_margin_end(12);
        self.set_margin_top(6);
        self.set_margin_bottom(6);

        let style_manager = adw::StyleManager::default();

        // System button
        let system_btn = gtk4::CheckButton::builder()
            .tooltip_text("Follow System Style")
            .halign(gtk4::Align::Center)
            .build();
        system_btn.add_css_class("theme-selector");
        system_btn.add_css_class("system");

        // Light button
        let light_btn = gtk4::CheckButton::builder()
            .tooltip_text("Light Style")
            .halign(gtk4::Align::Center)
            .group(&system_btn)
            .build();
        light_btn.add_css_class("theme-selector");
        light_btn.add_css_class("light");

        // Dark button
        let dark_btn = gtk4::CheckButton::builder()
            .tooltip_text("Dark Style")
            .halign(gtk4::Align::Center)
            .group(&system_btn)
            .build();
        dark_btn.add_css_class("theme-selector");
        dark_btn.add_css_class("dark");

        // Set initial state
        match style_manager.color_scheme() {
            adw::ColorScheme::ForceLight => light_btn.set_active(true),
            adw::ColorScheme::ForceDark => dark_btn.set_active(true),
            _ => system_btn.set_active(true),
        }

        // Connect signals
        let mgr = style_manager.clone();
        system_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                mgr.set_color_scheme(adw::ColorScheme::Default);
            }
        });

        let mgr = style_manager.clone();
        light_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                mgr.set_color_scheme(adw::ColorScheme::ForceLight);
            }
        });

        let mgr = style_manager;
        dark_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                mgr.set_color_scheme(adw::ColorScheme::ForceDark);
            }
        });

        self.append(&system_btn);
        self.append(&light_btn);
        self.append(&dark_btn);
    }
}
