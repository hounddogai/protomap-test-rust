use super::*;

use std::collections::BTreeMap;

use crate::identity::manifest as identity_manifest;

pub fn summarize_identity_reviews_for_tier(items: &[identity_manifest::IdentityReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_trust_domain_tiers_from_review(items: &[identity_manifest::IdentityReview]) -> Vec<TrustDomainTier> {
    let mut out: Vec<TrustDomainTier> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_trust_domain_tier(item: &TrustDomainTier) -> String {
    identity_manifest::classify_identity_manifest(item.measured_value())
}
