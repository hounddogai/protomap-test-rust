use super::*;

use std::collections::BTreeMap;

use crate::reservation::lenient_reservation_summary as reservation_lenient_reservation_summary;

pub fn summarize_lenient_reservation_allocations_for_summary(
    items: &[reservation_lenient_reservation_summary::LenientReservationAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_capped_overage_summaries_from_allocation(
    items: &[reservation_lenient_reservation_summary::LenientReservationAllocation],
) -> Vec<CappedOverageSummary> {
    let mut out: Vec<CappedOverageSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_overage_summary(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_capped_overage_summary(item: &CappedOverageSummary) -> f64 {
    reservation_lenient_reservation_summary::scheduled_reservation_summary_weight(9, true)
}
