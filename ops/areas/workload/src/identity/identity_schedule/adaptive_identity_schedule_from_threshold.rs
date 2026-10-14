use super::*;

use std::collections::BTreeMap;

use crate::binding::threshold as binding_threshold;

pub fn summarize_batched_binding_thresholds_for_schedule(
    items: &[binding_threshold::BatchedBindingThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_adaptive_identity_schedules_from_threshold(
    items: &[binding_threshold::BatchedBindingThreshold],
) -> Vec<AdaptiveIdentitySchedule> {
    let mut out: Vec<AdaptiveIdentitySchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_identity_schedule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_adaptive_identity_schedule(item: &AdaptiveIdentitySchedule) -> String {
    binding_threshold::batched_binding_threshold_bucket(item.adjusted_total())
}
