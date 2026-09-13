use super::*;

#[test]
fn charge_presets_use_standard_asus_thresholds() {
    assert_eq!(ChargeMode::FullCapacity.limit(), Some(100));
    assert_eq!(ChargeMode::Balanced.limit(), Some(80));
    assert_eq!(ChargeMode::MaxLifespan.limit(), Some(60));
    assert_eq!(ChargeMode::Custom.limit(), None);
}

#[test]
fn charge_mode_indices_follow_button_order() {
    for mode in ChargeMode::ALL {
        assert_eq!(ChargeMode::from_index(mode.index()), Some(mode));
    }
    assert_eq!(ChargeMode::from_index(gtk4::INVALID_LIST_POSITION), None);
}

#[test]
fn charge_limits_select_the_matching_mode() {
    assert_eq!(ChargeMode::from_limit(100), ChargeMode::FullCapacity);
    assert_eq!(ChargeMode::from_limit(80), ChargeMode::Balanced);
    assert_eq!(ChargeMode::from_limit(60), ChargeMode::MaxLifespan);

    for custom_limit in [20, 55, 75, 95, 99] {
        assert_eq!(ChargeMode::from_limit(custom_limit), ChargeMode::Custom);
    }
}

#[test]
fn refresh_keeps_an_explicit_custom_selection() {
    assert_eq!(
        ChargeMode::for_refreshed_limit(80, Some(ChargeMode::Custom)),
        ChargeMode::Custom
    );
    assert_eq!(
        ChargeMode::for_refreshed_limit(80, Some(ChargeMode::FullCapacity)),
        ChargeMode::Balanced
    );
}
