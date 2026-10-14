use super::*;

use std::collections::BTreeMap;

use crate::release::policy as release_policy;

pub fn summarize_release_estimates_for_claim(items: &[release_policy::ReleaseEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_rollback_claims_from_estimate(items: &[release_policy::ReleaseEstimate]) -> Vec<RollbackClaim> {
    let mut out: Vec<RollbackClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_claim(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_rollback_claim(item: &RollbackClaim) -> String {
    release_policy::classify_release_policy(item.adjusted_load())
}
