use super::*;

use std::collections::BTreeMap;

use crate::health_check::ledger as health_check_ledger;

pub fn summarize_adaptive_health_check_reviews_for_estimate(
    items: &[health_check_ledger::AdaptiveHealthCheckReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_fallback_delegation_estimates_from_review(
    items: &[health_check_ledger::AdaptiveHealthCheckReview],
) -> Vec<FallbackDelegationEstimate> {
    let mut out: Vec<FallbackDelegationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_delegation_estimate(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_fallback_delegation_estimate(item: &FallbackDelegationEstimate) -> String {
    health_check_ledger::normalize_health_check_target_code(&item.code)
}
