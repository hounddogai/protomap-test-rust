use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_snapshot as nonce_nonce_snapshot;

pub fn summarize_primary_nonce_scores_for_reservation(
    items: &[nonce_nonce_snapshot::PrimaryNonceScore],
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

pub fn derive_weighted_measurement_reservations_from_score(
    items: &[nonce_nonce_snapshot::PrimaryNonceScore],
) -> Vec<WeightedMeasurementReservation> {
    let mut out: Vec<WeightedMeasurementReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_measurement_reservation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_measurement_reservation(item: &WeightedMeasurementReservation) -> String {
    nonce_nonce_snapshot::classify_primary_nonce_score(item.projected_total())
}
