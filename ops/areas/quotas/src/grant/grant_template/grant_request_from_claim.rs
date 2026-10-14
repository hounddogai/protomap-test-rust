use super::*;

use std::collections::BTreeMap;

use crate::usage::claim as usage_claim;

pub fn summarize_adaptive_usage_claims_for_request(items: &[usage_claim::AdaptiveUsageClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_grant_requests_from_claim(items: &[usage_claim::AdaptiveUsageClaim]) -> Vec<GrantRequest> {
    let mut out: Vec<GrantRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_grant_request(item: &GrantRequest) -> String {
    usage_claim::classify_legacy_usage_assignment(item.net_total())
}
