use super::*;

const NOT_QUEUED: i32 = -1;

fn features(has_dgpu_disable: bool, has_gpu_mux: bool) -> SupportedFeatures {
    SupportedFeatures {
        has_dgpu_disable,
        has_gpu_mux,
        ..Default::default()
    }
}

// ------------------------------------------------------------------------
// available_gpu_modes
// ------------------------------------------------------------------------

#[test]
fn dgpu_and_mux_offer_all_modes() {
    assert_eq!(
        available_gpu_modes(&features(true, true)),
        vec![GpuMode::Integrated, GpuMode::Hybrid, GpuMode::Ultimate]
    );
}

#[test]
fn dgpu_without_mux_offers_integrated_and_hybrid() {
    assert_eq!(
        available_gpu_modes(&features(true, false)),
        vec![GpuMode::Integrated, GpuMode::Hybrid]
    );
}

#[test]
fn mux_without_dgpu_offers_hybrid_and_ultimate() {
    assert_eq!(
        available_gpu_modes(&features(false, true)),
        vec![GpuMode::Hybrid, GpuMode::Ultimate]
    );
}

#[test]
fn no_gpu_attributes_offer_no_modes() {
    assert!(available_gpu_modes(&features(false, false)).is_empty());
}

// ------------------------------------------------------------------------
// gpu_state_from_values
// ------------------------------------------------------------------------

#[test]
fn nothing_queued_has_no_pending_mode() {
    let state = gpu_state_from_values(Some((1, NOT_QUEUED)), Some((1, NOT_QUEUED)));
    assert_eq!(
        state,
        GpuState {
            active: GpuMode::Integrated,
            pending: None,
        }
    );
}

#[test]
fn queued_dgpu_enable_is_pending_hybrid() {
    let state = gpu_state_from_values(Some((1, 0)), Some((1, NOT_QUEUED)));
    assert_eq!(state.active, GpuMode::Integrated);
    assert_eq!(state.pending, Some(GpuMode::Hybrid));
}

#[test]
fn queued_pair_is_pending_ultimate() {
    let state = gpu_state_from_values(Some((1, 0)), Some((1, 0)));
    assert_eq!(state.active, GpuMode::Integrated);
    assert_eq!(state.pending, Some(GpuMode::Ultimate));
}

#[test]
fn queued_value_equal_to_current_is_not_pending() {
    let state = gpu_state_from_values(Some((0, 0)), Some((1, 1)));
    assert_eq!(state.active, GpuMode::Hybrid);
    assert_eq!(state.pending, None);
}

#[test]
fn mux_less_laptop_uses_dgpu_disable_only() {
    let state = gpu_state_from_values(Some((0, 1)), None);
    assert_eq!(state.active, GpuMode::Hybrid);
    assert_eq!(state.pending, Some(GpuMode::Integrated));
}

#[test]
fn dgpu_less_laptop_uses_mux_only() {
    let state = gpu_state_from_values(None, Some((0, NOT_QUEUED)));
    assert_eq!(state.active, GpuMode::Ultimate);
    assert_eq!(state.pending, None);
}

// ------------------------------------------------------------------------
// service_state_is_stopped
// ------------------------------------------------------------------------

#[test]
fn stopped_service_states() {
    for state in ["inactive", "failed", "deactivating"] {
        assert!(service_state_is_stopped(state), "expected stopped: {state}");
    }
}

#[test]
fn running_or_unknown_service_states_are_not_stopped() {
    // An empty or unknown state means systemd couldn't be asked; don't warn then.
    for state in ["active", "activating", "reloading", "refreshing", ""] {
        assert!(
            !service_state_is_stopped(state),
            "expected not stopped: {state}"
        );
    }
}
