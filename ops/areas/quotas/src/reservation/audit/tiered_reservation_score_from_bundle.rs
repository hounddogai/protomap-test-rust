use super::*;

use std::collections::BTreeMap;

use crate::reservation::bundle as reservation_bundle;

pub fn summarize_reservation_bundles_for_score(
    items: &[reservation_bundle::ReservationBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_tiered_reservation_scores_from_bundle(
    items: &[reservation_bundle::ReservationBundle],
) -> Vec<TieredReservationScore> {
    let mut out: Vec<TieredReservationScore> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_reservation_score(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_tiered_reservation_score(item: &TieredReservationScore) -> f64 {
    reservation_bundle::reservation_bundle_weight(2, true)
}
