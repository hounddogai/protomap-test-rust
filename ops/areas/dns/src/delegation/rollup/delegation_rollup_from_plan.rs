use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::policy_reservation as ttl_policy_policy_reservation;

pub fn summarize_primary_ttl_policy_plans_for_rollup(
    items: &[ttl_policy_policy_reservation::PrimaryTtlPolicyPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_delegation_rollups_from_plan(
    items: &[ttl_policy_policy_reservation::PrimaryTtlPolicyPlan],
) -> Vec<DelegationRollup> {
    let mut out: Vec<DelegationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_rollup(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_delegation_rollup(item: &DelegationRollup) -> String {
    ttl_policy_policy_reservation::classify_primary_ttl_policy_plan(item.weighted_volume())
}
