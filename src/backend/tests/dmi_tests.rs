use super::*;

// ------------------------------------------------------------------------
// sanitize_dmi_value Tests
// ------------------------------------------------------------------------

#[test]
fn test_sanitize_dmi_value_normal() {
    assert_eq!(sanitize_dmi_value("GA403UV.318"), "GA403UV.318");
}

#[test]
fn test_sanitize_dmi_value_trims_whitespace() {
    assert_eq!(sanitize_dmi_value("  1.P8 \n"), "1.P8");
}

#[test]
fn test_sanitize_dmi_value_placeholders() {
    for placeholder in [
        "Default string",
        "To Be Filled By O.E.M.",
        "System Serial Number",
        "Not Specified",
        "Not Applicable",
        "None",
        "Unknown",
        "N/A",
    ] {
        assert_eq!(
            sanitize_dmi_value(placeholder),
            "",
            "expected {placeholder} to be treated as empty"
        );
    }
}

#[test]
fn test_sanitize_dmi_value_placeholder_case_insensitive() {
    assert_eq!(sanitize_dmi_value("DEFAULT STRING\n"), "");
}

#[test]
fn test_sanitize_dmi_value_empty() {
    assert_eq!(sanitize_dmi_value(""), "");
    assert_eq!(sanitize_dmi_value("   \n"), "");
}

// ------------------------------------------------------------------------
// format_bios Tests
// ------------------------------------------------------------------------

fn bios_info(version: &str, date: &str) -> DmiInfo {
    DmiInfo {
        bios_version: version.to_string(),
        bios_date: date.to_string(),
        ..DmiInfo::default()
    }
}

#[test]
fn test_format_bios_version_and_date() {
    assert_eq!(
        format_bios(&bios_info("1.P8", "03/17/2026")),
        "1.P8 · 2026-03-17"
    );
}

#[test]
fn test_format_bios_version_only() {
    assert_eq!(format_bios(&bios_info("1.P8", "")), "1.P8");
}

#[test]
fn test_format_bios_date_only() {
    assert_eq!(format_bios(&bios_info("", "03/17/2026")), "2026-03-17");
}

#[test]
fn test_format_bios_neither() {
    assert_eq!(format_bios(&bios_info("", "")), "Unknown");
}

#[test]
fn test_format_bios_keeps_unparsable_date() {
    assert_eq!(
        format_bios(&bios_info("1.P8", "17.03.2026")),
        "1.P8 · 17.03.2026"
    );
}

// ------------------------------------------------------------------------
// to_iso_date Tests
// ------------------------------------------------------------------------

#[test]
fn test_to_iso_date_smbios_format() {
    assert_eq!(to_iso_date("07/02/2026").as_deref(), Some("2026-07-02"));
}

#[test]
fn test_to_iso_date_pads_single_digits() {
    assert_eq!(to_iso_date("7/2/2026").as_deref(), Some("2026-07-02"));
}

#[test]
fn test_to_iso_date_rejects_out_of_range() {
    assert_eq!(to_iso_date("13/02/2026"), None);
    assert_eq!(to_iso_date("07/32/2026"), None);
}

#[test]
fn test_to_iso_date_rejects_other_formats() {
    assert_eq!(to_iso_date("17.03.2026"), None);
    assert_eq!(to_iso_date("2026-03-17"), None);
    assert_eq!(to_iso_date("07/02/2026/1"), None);
    assert_eq!(to_iso_date(""), None);
}

// ------------------------------------------------------------------------
// interpret_pkexec_failure Tests
// ------------------------------------------------------------------------

#[test]
fn test_interpret_pkexec_failure_not_authorized() {
    let message = interpret_pkexec_failure(Some(126)).to_string();
    assert!(message.contains("Authorization"), "got: {message}");
}

#[test]
fn test_interpret_pkexec_failure_missing_pkexec() {
    let message = interpret_pkexec_failure(Some(127)).to_string();
    assert!(
        message.contains("pkexec is not available"),
        "got: {message}"
    );
}

#[test]
fn test_interpret_pkexec_failure_other_code() {
    let message = interpret_pkexec_failure(Some(1)).to_string();
    assert!(message.contains("code 1"), "got: {message}");
}

#[test]
fn test_interpret_pkexec_failure_signal() {
    let message = interpret_pkexec_failure(None).to_string();
    assert!(message.contains("signal"), "got: {message}");
}
