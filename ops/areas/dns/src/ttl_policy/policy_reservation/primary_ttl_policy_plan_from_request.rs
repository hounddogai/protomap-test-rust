use super::*;

use std::collections::BTreeMap;

use crate::delegation::draft as delegation_draft;

pub fn summarize_tiered_delegation_requests_for_plan(
    items: &[delegation_draft::TieredDelegationRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_primary_ttl_policy_plans_from_request(
    items: &[delegation_draft::TieredDelegationRequest],
) -> Vec<PrimaryTtlPolicyPlan> {
    let mut out: Vec<PrimaryTtlPolicyPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_ttl_policy_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_ttl_policy_plan_extra(item: &PrimaryTtlPolicyPlan) -> String {
    delegation_draft::normalize_tiered_delegation_request_code(&item.code)
}
