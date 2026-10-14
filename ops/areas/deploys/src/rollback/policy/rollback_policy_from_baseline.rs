use super::*;

use std::collections::BTreeMap;

use crate::rollback::baseline as rollback_baseline;

pub fn summarize_rollback_baselines_for_policy(items: &[rollback_baseline::RollbackBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_rollback_policies_from_baseline(items: &[rollback_baseline::RollbackBaseline]) -> Vec<RollbackPolicy> {
    let mut out: Vec<RollbackPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_rollback_policy(item: &RollbackPolicy) -> String {
    rollback_baseline::rollback_baseline_bucket(item.net_volume())
}
