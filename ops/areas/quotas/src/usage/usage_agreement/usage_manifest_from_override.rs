use super::*;

use std::collections::BTreeMap;

use crate::reservation::review as reservation_review;

pub fn summarize_scheduled_reservation_overrides_for_manifest(
    items: &[reservation_review::ScheduledReservationOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_usage_manifests_from_override(
    items: &[reservation_review::ScheduledReservationOverride],
) -> Vec<UsageManifest> {
    let mut out: Vec<UsageManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_usage_manifest(item: &UsageManifest) -> String {
    reservation_review::classify_primary_reservation_review(item.weighted_total())
}
