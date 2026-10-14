use super::*;

use std::collections::BTreeMap;

use crate::rollout::override_item as rollout_override_item;

pub fn summarize_primary_rollout_overrides_for_record(
    items: &[rollout_override_item::PrimaryRolloutOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_canary_records_from_override(
    items: &[rollout_override_item::PrimaryRolloutOverride],
) -> Vec<CanaryRecord> {
    let mut out: Vec<CanaryRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_record(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_canary_record(item: &CanaryRecord) -> f64 {
    rollout_override_item::rollout_threshold_weight(1, true)
}
