use super::*;

const CUSTOM0: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/";
const CUSTOM1: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom1/";

#[test]
fn parse_path_list_reads_gsettings_output() {
    let text = format!("['{CUSTOM0}', '{CUSTOM1}']");
    assert_eq!(
        parse_path_list(&text).unwrap(),
        vec![CUSTOM0.to_string(), CUSTOM1.to_string()]
    );
}

#[test]
fn parse_path_list_reads_empty_list() {
    assert!(parse_path_list("@as []").unwrap().is_empty());
}

#[test]
fn parse_path_list_rejects_garbage() {
    assert!(matches!(
        parse_path_list("No such schema"),
        Err(AsusctlError::ParseError(_))
    ));
}

#[test]
fn parse_string_unquotes_value() {
    assert_eq!(parse_string("'Launch1'").unwrap(), "Launch1");
    assert_eq!(
        parse_string(r#"'ptyxis -- bash -ic "update-all; exec $SHELL" '"#).unwrap(),
        r#"ptyxis -- bash -ic "update-all; exec $SHELL" "#
    );
}

#[test]
fn format_path_list_round_trips() {
    let paths = vec![CUSTOM0.to_string(), SHORTCUT_PATH.to_string()];
    assert_eq!(parse_path_list(&format_path_list(&paths)).unwrap(), paths);
    assert!(parse_path_list(&format_path_list(&[])).unwrap().is_empty());
}

#[test]
fn quote_round_trips_special_characters() {
    let value = "it's \"quoted\"";
    assert_eq!(parse_string(&quote(value)).unwrap(), value);
}

#[test]
fn has_shortcut_matches_only_the_app_path() {
    assert!(has_shortcut(&[
        CUSTOM0.to_string(),
        SHORTCUT_PATH.to_string()
    ]));
    assert!(!has_shortcut(&[CUSTOM0.to_string(), CUSTOM1.to_string()]));
    assert!(!has_shortcut(&[]));
}

#[test]
fn custom_schema_targets_the_app_path() {
    assert_eq!(
        custom_schema(),
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:\
         /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/asusctl-gui/"
    );
}
