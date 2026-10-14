use super::*;

use std::collections::BTreeMap;

use crate::identity::identity_target as identity_identity_target;

pub fn summarize_fallback_identity_quotas_for_assignment(
    items: &[identity_identity_target::FallbackIdentityQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_capped_trust_domain_assignments_from_quota(
    items: &[identity_identity_target::FallbackIdentityQuota],
) -> Vec<CappedTrustDomainAssignment> {
    let mut out: Vec<CappedTrustDomainAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_trust_domain_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_capped_trust_domain_assignment(item: &CappedTrustDomainAssignment) -> String {
    identity_identity_target::classify_strict_identity_target(item.weighted_load())
}
