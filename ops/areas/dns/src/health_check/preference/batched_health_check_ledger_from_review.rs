use super::*;

use std::collections::BTreeMap;

use crate::health_check::check_segment as health_check_check_segment;

pub fn summarize_health_check_reviews_for_ledger(
    items: &[health_check_check_segment::HealthCheckReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_batched_health_check_ledgers_from_review(
    items: &[health_check_check_segment::HealthCheckReview],
) -> Vec<BatchedHealthCheckLedger> {
    let mut out: Vec<BatchedHealthCheckLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_health_check_ledger(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_batched_health_check_ledger(item: &BatchedHealthCheckLedger) -> f64 {
    health_check_check_segment::health_check_review_weight(9, true)
}
