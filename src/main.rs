mod app;
mod pages;
mod window;

use gtk4::prelude::*;

fn main() -> gtk4::glib::ExitCode {
    let app = app::AsusctlGuiApp::new();
    app.run()
}
