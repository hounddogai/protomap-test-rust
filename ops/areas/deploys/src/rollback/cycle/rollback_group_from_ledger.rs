use super::*;

use std::collections::BTreeMap;

use crate::rollout::window as rollout_window;

pub fn summarize_rollout_ledgers_for_group(items: &[rollout_window::RolloutLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_rollback_groups_from_ledger(items: &[rollout_window::RolloutLedger]) -> Vec<RollbackGroup> {
    let mut out: Vec<RollbackGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_rollback_group(item: &RollbackGroup) -> f64 {
    rollout_window::rollout_cycle_weight(5, true)
}
