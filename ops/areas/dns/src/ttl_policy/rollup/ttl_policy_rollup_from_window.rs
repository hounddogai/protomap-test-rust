use super::*;

use std::collections::BTreeMap;

use crate::delegation::proposal as delegation_proposal;

pub fn summarize_delegation_windows_for_rollup(
    items: &[delegation_proposal::DelegationWindow],
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

pub fn derive_ttl_policy_rollups_from_window(items: &[delegation_proposal::DelegationWindow]) -> Vec<TtlPolicyRollup> {
    let mut out: Vec<TtlPolicyRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_rollup(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_ttl_policy_rollup(item: &TtlPolicyRollup) -> String {
    delegation_proposal::classify_delegation_manifest(item.weighted_load())
}
