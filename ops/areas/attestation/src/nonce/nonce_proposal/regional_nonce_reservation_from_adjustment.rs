use super::*;

use std::collections::BTreeMap;

use crate::measurement::segment as measurement_segment;

pub fn summarize_measurement_adjustments_for_reservation(
    items: &[measurement_segment::MeasurementAdjustment],
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

pub fn derive_regional_nonce_reservations_from_adjustment(
    items: &[measurement_segment::MeasurementAdjustment],
) -> Vec<RegionalNonceReservation> {
    let mut out: Vec<RegionalNonceReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_nonce_reservation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_regional_nonce_reservation(item: &RegionalNonceReservation) -> String {
    measurement_segment::classify_measurement_adjustment(item.adjusted_volume())
}
