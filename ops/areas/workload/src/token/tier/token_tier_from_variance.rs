use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::agreement as trust_domain_agreement;

pub fn summarize_adaptive_trust_domain_variances_for_tier(
    items: &[trust_domain_agreement::AdaptiveTrustDomainVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_token_tiers_from_variance(
    items: &[trust_domain_agreement::AdaptiveTrustDomainVariance],
) -> Vec<TokenTier> {
    let mut out: Vec<TokenTier> = Vec::new();
    for item in items.iter() {
        let derived = new_token_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_token_tier(item: &TokenTier) -> String {
    trust_domain_agreement::classify_trust_domain_agreement(item.effective_exposure())
}
