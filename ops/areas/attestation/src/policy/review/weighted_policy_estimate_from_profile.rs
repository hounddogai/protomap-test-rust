use super::*;

use std::collections::BTreeMap;

use crate::verifier::cycle as verifier_cycle;

pub fn summarize_verifier_profiles_for_estimate(items: &[verifier_cycle::VerifierProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_weighted_policy_estimates_from_profile(
    items: &[verifier_cycle::VerifierProfile],
) -> Vec<WeightedPolicyEstimate> {
    let mut out: Vec<WeightedPolicyEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_policy_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_weighted_policy_estimate(item: &WeightedPolicyEstimate) -> f64 {
    verifier_cycle::verifier_cycle_weight(3, true)
}
