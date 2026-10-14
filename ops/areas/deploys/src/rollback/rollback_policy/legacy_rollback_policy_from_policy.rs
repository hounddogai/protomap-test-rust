use super::*;

use std::collections::BTreeMap;

use crate::rollback::policy as rollback_policy;

pub fn summarize_rollback_policies_for_policy(items: &[rollback_policy::RollbackPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_legacy_rollback_policies_from_policy(
    items: &[rollback_policy::RollbackPolicy],
) -> Vec<LegacyRollbackPolicy> {
    let mut out: Vec<LegacyRollbackPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_rollback_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_rollback_policy(item: &LegacyRollbackPolicy) -> f64 {
    rollback_policy::rollback_policy_weight(3, true)
}
