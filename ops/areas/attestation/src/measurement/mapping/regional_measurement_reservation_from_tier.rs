use super::*;

use std::collections::BTreeMap;

use crate::policy::reservation as policy_reservation;

pub fn summarize_batched_policy_tiers_for_reservation(
    items: &[policy_reservation::BatchedPolicyTier],
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

pub fn derive_regional_measurement_reservations_from_tier(
    items: &[policy_reservation::BatchedPolicyTier],
) -> Vec<RegionalMeasurementReservation> {
    let mut out: Vec<RegionalMeasurementReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_measurement_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_regional_measurement_reservation(item: &RegionalMeasurementReservation) -> String {
    policy_reservation::normalize_pending_policy_variance_code(&item.code)
}
