use super::*;

use std::collections::BTreeMap;

use crate::measurement::cycle as measurement_cycle;

pub fn summarize_measurement_tiers_for_score(items: &[measurement_cycle::MeasurementTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_scheduled_nonce_scores_from_tier(
    items: &[measurement_cycle::MeasurementTier],
) -> Vec<ScheduledNonceScore> {
    let mut out: Vec<ScheduledNonceScore> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_nonce_score(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_scheduled_nonce_score(item: &ScheduledNonceScore) -> String {
    measurement_cycle::pending_measurement_cycle_bucket(item.measured_value())
}
