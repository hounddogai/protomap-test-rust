use super::*;

use std::collections::BTreeMap;

use crate::rollout::window as rollout_window;

pub fn summarize_rollout_ledgers_for_assignment(items: &[rollout_window::RolloutLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_release_assignments_from_ledger(items: &[rollout_window::RolloutLedger]) -> Vec<ReleaseAssignment> {
    let mut out: Vec<ReleaseAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_release_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_release_assignment(item: &ReleaseAssignment) -> String {
    rollout_window::normalize_rollout_window_code(&item.code)
}
