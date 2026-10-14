use super::*;

use std::collections::BTreeMap;

use crate::limit::allocation as limit_allocation;

pub fn summarize_limit_allocations_for_summary(items: &[limit_allocation::LimitAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_reservation_summaries_from_allocation(
    items: &[limit_allocation::LimitAllocation],
) -> Vec<ReservationSummary> {
    let mut out: Vec<ReservationSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_reservation_summary(item: &ReservationSummary) -> String {
    limit_allocation::limit_claim_bucket(item.net_exposure())
}
