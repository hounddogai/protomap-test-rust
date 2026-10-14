use super::*;

use std::collections::BTreeMap;

use crate::failover::exception as failover_exception;

pub fn summarize_adaptive_failover_exceptions_for_review(
    items: &[failover_exception::AdaptiveFailoverException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_health_check_reviews_from_exception(
    items: &[failover_exception::AdaptiveFailoverException],
) -> Vec<HealthCheckReview> {
    let mut out: Vec<HealthCheckReview> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_review(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_health_check_review(item: &HealthCheckReview) -> f64 {
    failover_exception::pending_failover_segment_weight(5, true)
}
