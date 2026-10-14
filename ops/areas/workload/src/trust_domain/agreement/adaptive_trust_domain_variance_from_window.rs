use super::*;

use std::collections::BTreeMap;

use crate::token::draft as token_draft;

pub fn summarize_token_windows_for_variance(items: &[token_draft::TokenWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_adaptive_trust_domain_variances_from_window(
    items: &[token_draft::TokenWindow],
) -> Vec<AdaptiveTrustDomainVariance> {
    let mut out: Vec<AdaptiveTrustDomainVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_trust_domain_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_adaptive_trust_domain_variance(item: &AdaptiveTrustDomainVariance) -> f64 {
    token_draft::token_window_weight(3, true)
}
