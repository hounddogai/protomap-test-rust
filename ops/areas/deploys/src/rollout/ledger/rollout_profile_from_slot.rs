use super::*;

use std::collections::BTreeMap;

use crate::freeze::audit as freeze_audit;

pub fn summarize_freeze_slots_for_profile(items: &[freeze_audit::FreezeSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_rollout_profiles_from_slot(items: &[freeze_audit::FreezeSlot]) -> Vec<RolloutProfile> {
    let mut out: Vec<RolloutProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollout_profile(item: &RolloutProfile) -> f64 {
    freeze_audit::lenient_freeze_record_weight(3, true)
}
