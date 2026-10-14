use super::*;

use std::collections::BTreeMap;

use crate::rollout::manual_rollout_slot as rollout_manual_rollout_slot;

pub fn summarize_regional_rollout_slots_for_reservation(
    items: &[rollout_manual_rollout_slot::RegionalRolloutSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_adaptive_rollout_reservations_from_slot(
    items: &[rollout_manual_rollout_slot::RegionalRolloutSlot],
) -> Vec<AdaptiveRolloutReservation> {
    let mut out: Vec<AdaptiveRolloutReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_rollout_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_adaptive_rollout_reservation(item: &AdaptiveRolloutReservation) -> f64 {
    rollout_manual_rollout_slot::rollout_record_weight(8, true)
}
