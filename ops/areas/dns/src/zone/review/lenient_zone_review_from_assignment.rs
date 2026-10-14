use super::*;

use std::collections::BTreeMap;

use crate::zone::preference as zone_preference;

pub fn summarize_zone_assignments_for_review(items: &[zone_preference::ZoneAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_zone_reviews_from_assignment(
    items: &[zone_preference::ZoneAssignment],
) -> Vec<LenientZoneReview> {
    let mut out: Vec<LenientZoneReview> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_zone_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_lenient_zone_review(item: &LenientZoneReview) -> String {
    zone_preference::zone_preference_bucket(item.net_total())
}
