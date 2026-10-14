use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_draft as delegation_delegation_draft;

pub fn summarize_delegation_metrics_for_estimate(
    items: &[delegation_delegation_draft::DelegationMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_regional_ttl_policy_estimates_from_metric(
    items: &[delegation_delegation_draft::DelegationMetric],
) -> Vec<RegionalTtlPolicyEstimate> {
    let mut out: Vec<RegionalTtlPolicyEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_ttl_policy_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_regional_ttl_policy_estimate(item: &RegionalTtlPolicyEstimate) -> f64 {
    delegation_delegation_draft::tiered_delegation_claim_weight(3, true)
}
