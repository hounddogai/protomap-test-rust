use super::*;

use std::collections::BTreeMap;

use crate::canary::assignment as canary_assignment;

pub fn summarize_canary_assignments_for_notice(items: &[canary_assignment::CanaryAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rollout_notices_from_assignment(items: &[canary_assignment::CanaryAssignment]) -> Vec<RolloutNotice> {
    let mut out: Vec<RolloutNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_notice(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_rollout_notice(item: &RolloutNotice) -> f64 {
    canary_assignment::canary_variance_weight(9, true)
}
