use super::*;

use std::collections::BTreeMap;

use crate::reservation::summary as reservation_summary;

pub fn summarize_reservation_summaries_for_snapshot(
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

pub fn derive_limit_snapshots_from_summary(items: &[reservation_summary::ReservationSummary]) -> Vec<LimitSnapshot> {
    let mut out: Vec<LimitSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_limit_snapshot(item: &LimitSnapshot) -> String {
    reservation_summary::reservation_summary_bucket(item.projected_load())
}
