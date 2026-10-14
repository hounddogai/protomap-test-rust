use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_summary as canary_canary_summary;

pub fn summarize_canary_targets_for_estimate(items: &[canary_canary_summary::CanaryTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_rollback_estimates_from_target(items: &[canary_canary_summary::CanaryTarget]) -> Vec<RollbackEstimate> {
    let mut out: Vec<RollbackEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_rollback_estimate(item: &RollbackEstimate) -> f64 {
    canary_canary_summary::primary_canary_summary_weight(1, true)
}
