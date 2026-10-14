use super::*;

use std::collections::BTreeMap;

use crate::failover::summary as failover_summary;

pub fn summarize_strict_failover_snapshots_for_target(
    items: &[failover_summary::StrictFailoverSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_delegation_targets_from_snapshot(
    items: &[failover_summary::StrictFailoverSnapshot],
) -> Vec<DelegationTarget> {
    let mut out: Vec<DelegationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_target(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_delegation_target(item: &DelegationTarget) -> String {
    failover_summary::classify_manual_failover_baseline(item.effective_total())
}
