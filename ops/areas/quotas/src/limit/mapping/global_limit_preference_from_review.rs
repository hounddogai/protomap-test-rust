use super::*;

use std::collections::BTreeMap;

use crate::reservation::review as reservation_review;

pub fn summarize_primary_reservation_reviews_for_preference(
    items: &[reservation_review::PrimaryReservationReview],
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

pub fn derive_global_limit_preferences_from_review(
    items: &[reservation_review::PrimaryReservationReview],
) -> Vec<GlobalLimitPreference> {
    let mut out: Vec<GlobalLimitPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_global_limit_preference(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_global_limit_preference(item: &GlobalLimitPreference) -> String {
    reservation_review::classify_strict_reservation_segment(item.projected_volume())
}
