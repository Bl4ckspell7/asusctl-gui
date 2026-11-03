use super::*;

fn all_off() -> AuraPowerState {
    AuraPowerState {
        zone: AuraPowerZone::Keyboard,
        boot: false,
        awake: false,
        sleep: false,
        shutdown: false,
    }
}

#[test]
fn power_state_rows_match_their_titles() {
    for entry in &POWER_STATE_ROWS {
        let mut state = all_off();
        (entry.set)(&mut state, true);
        let field = match entry.title {
            "Awake" => state.awake,
            "Boot" => state.boot,
            "Sleep" => state.sleep,
            "Shutdown" => state.shutdown,
            other => panic!("unexpected power state row: {other}"),
        };
        assert!(field, "{} sets the wrong field", entry.title);
        assert!((entry.get)(&state), "{} reads the wrong field", entry.title);
    }
}

#[test]
fn power_state_rows_each_change_only_their_own_field() {
    for (index, entry) in POWER_STATE_ROWS.iter().enumerate() {
        let mut state = all_off();
        (entry.set)(&mut state, true);
        for (other_index, other) in POWER_STATE_ROWS.iter().enumerate() {
            assert_eq!(
                (other.get)(&state),
                index == other_index,
                "setting {} changed {}",
                entry.title,
                other.title
            );
        }
    }
}
