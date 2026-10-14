use super::*;

use std::collections::BTreeMap;

use crate::limit::budget as limit_budget;

pub fn summarize_manual_limit_allocations_for_summary(
    items: &[limit_budget::ManualLimitAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_scheduled_reservation_summaries_from_allocation(
    items: &[limit_budget::ManualLimitAllocation],
) -> Vec<ScheduledReservationSummary> {
    let mut out: Vec<ScheduledReservationSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_reservation_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_scheduled_reservation_summary(item: &ScheduledReservationSummary) -> String {
    limit_budget::global_limit_digest_bucket(item.weighted_total())
}
