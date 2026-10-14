use super::*;

use std::collections::BTreeMap;

use crate::policy::policy_digest as policy_policy_digest;

pub fn summarize_policy_baselines_for_policy(items: &[policy_policy_digest::PolicyBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_nonce_policies_from_baseline(items: &[policy_policy_digest::PolicyBaseline]) -> Vec<NoncePolicy> {
    let mut out: Vec<NoncePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_policy(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_nonce_policy(item: &NoncePolicy) -> f64 {
    policy_policy_digest::policy_allocation_weight(3, true)
}
