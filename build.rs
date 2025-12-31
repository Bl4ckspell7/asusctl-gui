fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/asusctl-gui.gresource.xml",
        "asusctl-gui.gresource",
    );
}
