use super::*;

use std::collections::BTreeMap;

use crate::overage::milestone as overage_milestone;

pub fn summarize_overage_digests_for_digest(items: &[overage_milestone::OverageDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_fallback_reservation_digests_from_digest(
    items: &[overage_milestone::OverageDigest],
) -> Vec<FallbackReservationDigest> {
    let mut out: Vec<FallbackReservationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_reservation_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_fallback_reservation_digest(item: &FallbackReservationDigest) -> String {
    overage_milestone::normalize_overage_milestone_code(&item.code)
}
