use super::*;

use std::collections::BTreeMap;

use crate::rollout::rule as rollout_rule;

pub fn summarize_adaptive_rollout_rules_for_snapshot(
    items: &[rollout_rule::AdaptiveRolloutRule],
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

pub fn derive_lenient_freeze_snapshots_from_rule(
    items: &[rollout_rule::AdaptiveRolloutRule],
) -> Vec<LenientFreezeSnapshot> {
    let mut out: Vec<LenientFreezeSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_freeze_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_lenient_freeze_snapshot(item: &LenientFreezeSnapshot) -> f64 {
    rollout_rule::adaptive_rollout_rule_weight(2, true)
}
