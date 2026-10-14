use super::*;

use std::collections::BTreeMap;

use crate::rollout::override_item as rollout_override_item;

pub fn summarize_rollout_thresholds_for_adjustment(
    items: &[rollout_override_item::RolloutThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_tiered_approval_adjustments_from_threshold(
    items: &[rollout_override_item::RolloutThreshold],
) -> Vec<TieredApprovalAdjustment> {
    let mut out: Vec<TieredApprovalAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_approval_adjustment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_tiered_approval_adjustment(item: &TieredApprovalAdjustment) -> f64 {
    rollout_override_item::primary_rollout_override_weight(2, true)
}
