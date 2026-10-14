use super::*;

use std::collections::BTreeMap;

use crate::evidence::score as evidence_score;

pub fn summarize_weighted_evidence_reservations_for_reservation(
    items: &[evidence_score::WeightedEvidenceReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_regional_measurement_reservations_from_reservation(
    items: &[evidence_score::WeightedEvidenceReservation],
) -> Vec<RegionalMeasurementReservation> {
    let mut out: Vec<RegionalMeasurementReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_measurement_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_regional_measurement_reservation(item: &RegionalMeasurementReservation) -> String {
    evidence_score::rolling_evidence_slot_bucket(item.projected_load())
}
