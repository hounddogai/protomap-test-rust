use super::*;

use std::collections::BTreeMap;

use crate::reservation::reservation_summary as reservation_reservation_summary;

pub fn summarize_reservation_milestones_for_threshold(
    items: &[reservation_reservation_summary::ReservationMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_global_overage_thresholds_from_milestone(
    items: &[reservation_reservation_summary::ReservationMilestone],
) -> Vec<GlobalOverageThreshold> {
    let mut out: Vec<GlobalOverageThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_global_overage_threshold(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_global_overage_threshold(item: &GlobalOverageThreshold) -> String {
    reservation_reservation_summary::normalize_reservation_digest_code(&item.code)
}
