use super::*;

use std::collections::BTreeMap;

use crate::identity::group as identity_group;

pub fn summarize_identity_groups_for_quota(items: &[identity_group::IdentityGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_trust_domain_quotas_from_group(items: &[identity_group::IdentityGroup]) -> Vec<TrustDomainQuota> {
    let mut out: Vec<TrustDomainQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_trust_domain_quota(item: &TrustDomainQuota) -> String {
    identity_group::identity_group_bucket(item.measured_value())
}
