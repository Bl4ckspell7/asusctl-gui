use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;

use crate::window::AsusctlGuiWindow;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct AsusctlGuiApp;

    #[glib::object_subclass]
    impl ObjectSubclass for AsusctlGuiApp {
        const NAME: &'static str = "AsusctlGuiApp";
        type Type = super::AsusctlGuiApp;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for AsusctlGuiApp {}

    impl ApplicationImpl for AsusctlGuiApp {
        fn activate(&self) {
            let obj = self.obj();
            let app: &adw::Application = obj.upcast_ref();

            // Set up keyboard shortcuts
            app.set_accels_for_action("win.quit", &["<Control>q"]);
            app.set_accels_for_action("win.show-shortcuts", &["<Control>question"]);

            let window = AsusctlGuiWindow::new(app);
            window.present();
        }
    }

    impl GtkApplicationImpl for AsusctlGuiApp {}
    impl AdwApplicationImpl for AsusctlGuiApp {}
}

glib::wrapper! {
    pub struct AsusctlGuiApp(ObjectSubclass<imp::AsusctlGuiApp>)
        @extends adw::Application, gtk4::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl AsusctlGuiApp {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "com.github.bl4ckspell7.asusctl-gui")
            .build()
    }
}
