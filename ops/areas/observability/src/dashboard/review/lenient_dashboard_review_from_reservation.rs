use super::*;

use std::collections::BTreeMap;

use crate::dashboard::rule as dashboard_rule;

pub fn summarize_primary_dashboard_reservations_for_review(
    items: &[dashboard_rule::PrimaryDashboardReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_lenient_dashboard_reviews_from_reservation(
    items: &[dashboard_rule::PrimaryDashboardReservation],
) -> Vec<LenientDashboardReview> {
    let mut out: Vec<LenientDashboardReview> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_dashboard_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_dashboard_review(item: &LenientDashboardReview) -> String {
    dashboard_rule::classify_primary_dashboard_reservation(item.measured_value())
}
