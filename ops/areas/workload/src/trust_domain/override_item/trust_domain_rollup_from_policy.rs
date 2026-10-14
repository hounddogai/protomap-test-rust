use super::*;

use std::collections::BTreeMap;

use crate::identity::policy as identity_policy;

pub fn summarize_identity_policies_for_rollup(items: &[identity_policy::IdentityPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_trust_domain_rollups_from_policy(items: &[identity_policy::IdentityPolicy]) -> Vec<TrustDomainRollup> {
    let mut out: Vec<TrustDomainRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_rollup(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_trust_domain_rollup(item: &TrustDomainRollup) -> String {
    identity_policy::normalize_identity_policy_code(&item.code)
}
