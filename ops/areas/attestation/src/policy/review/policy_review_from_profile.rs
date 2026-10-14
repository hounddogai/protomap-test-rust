use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_profile as nonce_nonce_profile;

pub fn summarize_adaptive_nonce_profiles_for_review(
    items: &[nonce_nonce_profile::AdaptiveNonceProfile],
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

pub fn derive_policy_reviews_from_profile(items: &[nonce_nonce_profile::AdaptiveNonceProfile]) -> Vec<PolicyReview> {
    let mut out: Vec<PolicyReview> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_policy_review(item: &PolicyReview) -> String {
    nonce_nonce_profile::nonce_notice_bucket(item.weighted_total())
}
