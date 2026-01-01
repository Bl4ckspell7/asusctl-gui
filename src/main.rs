mod app;
mod backend;
mod ui;

use gtk4::gio;
use gtk4::prelude::*;

fn main() -> gtk4::glib::ExitCode {
    // Register resources (this is fine before init)
    gio::resources_register_include!("asusctl-gui.gresource")
        .expect("Failed to register resources.");

    let app = app::AsusctlGuiApp::new();

    // Load CSS after GTK is initialized (on startup)
    app.connect_startup(|_| {
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_resource("/com/github/bl4ckspell7/asusctl-gui/style.css");

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not get default display"),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.run()
}
