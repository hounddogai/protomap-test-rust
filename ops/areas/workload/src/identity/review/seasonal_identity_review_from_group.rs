use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::threshold as trust_domain_threshold;

pub fn summarize_trust_domain_groups_for_review(
    items: &[trust_domain_threshold::TrustDomainGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_seasonal_identity_reviews_from_group(
    items: &[trust_domain_threshold::TrustDomainGroup],
) -> Vec<SeasonalIdentityReview> {
    let mut out: Vec<SeasonalIdentityReview> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_identity_review(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_seasonal_identity_review(item: &SeasonalIdentityReview) -> f64 {
    trust_domain_threshold::trust_domain_threshold_weight(3, true)
}
