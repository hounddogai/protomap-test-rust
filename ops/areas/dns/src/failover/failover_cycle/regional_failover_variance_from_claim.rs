use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::budget as ttl_policy_budget;

pub fn summarize_ttl_policy_claims_for_variance(items: &[ttl_policy_budget::TtlPolicyClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_regional_failover_variances_from_claim(
    items: &[ttl_policy_budget::TtlPolicyClaim],
) -> Vec<RegionalFailoverVariance> {
    let mut out: Vec<RegionalFailoverVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_failover_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_regional_failover_variance(item: &RegionalFailoverVariance) -> String {
    ttl_policy_budget::classify_ttl_policy_claim(item.projected_load())
}
