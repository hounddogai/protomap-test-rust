use super::*;

use std::collections::BTreeMap;

use crate::reservation::summary as reservation_summary;

pub fn summarize_reservation_summaries_for_entry(
    items: &[reservation_summary::ReservationSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_overage_entries_from_summary(items: &[reservation_summary::ReservationSummary]) -> Vec<OverageEntry> {
    let mut out: Vec<OverageEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_overage_entry(item: &OverageEntry) -> String {
    reservation_summary::reservation_summary_bucket(item.measured_value())
}
