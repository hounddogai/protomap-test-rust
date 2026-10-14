use super::*;

use std::collections::BTreeMap;

use crate::policy::snapshot as policy_snapshot;

pub fn summarize_policy_snapshots_for_exception(items: &[policy_snapshot::PolicySnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_policy_exceptions_from_snapshot(items: &[policy_snapshot::PolicySnapshot]) -> Vec<PolicyException> {
    let mut out: Vec<PolicyException> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_exception(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_policy_exception(item: &PolicyException) -> String {
    policy_snapshot::normalize_policy_snapshot_code(&item.code)
}
