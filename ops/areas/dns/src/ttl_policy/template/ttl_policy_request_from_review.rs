use super::*;

use std::collections::BTreeMap;

use crate::zone::review as zone_review;

pub fn summarize_legacy_zone_reviews_for_request(items: &[zone_review::LegacyZoneReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_ttl_policy_requests_from_review(items: &[zone_review::LegacyZoneReview]) -> Vec<TtlPolicyRequest> {
    let mut out: Vec<TtlPolicyRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_request(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_ttl_policy_request(item: &TtlPolicyRequest) -> f64 {
    zone_review::legacy_zone_review_weight(2, true)
}
