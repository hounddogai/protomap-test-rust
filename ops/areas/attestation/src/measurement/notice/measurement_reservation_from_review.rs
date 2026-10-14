use super::*;

use std::collections::BTreeMap;

use crate::evidence::cycle as evidence_cycle;

pub fn summarize_evidence_reviews_for_reservation(items: &[evidence_cycle::EvidenceReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_measurement_reservations_from_review(
    items: &[evidence_cycle::EvidenceReview],
) -> Vec<MeasurementReservation> {
    let mut out: Vec<MeasurementReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_measurement_reservation(item: &MeasurementReservation) -> f64 {
    evidence_cycle::evidence_review_weight(3, true)
}
