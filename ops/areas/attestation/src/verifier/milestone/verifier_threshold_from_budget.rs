use super::*;

use std::collections::BTreeMap;

use crate::measurement::review as measurement_review;

pub fn summarize_measurement_budgets_for_threshold(
    items: &[measurement_review::MeasurementBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_verifier_thresholds_from_budget(
    items: &[measurement_review::MeasurementBudget],
) -> Vec<VerifierThreshold> {
    let mut out: Vec<VerifierThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_verifier_threshold(item: &VerifierThreshold) -> String {
    measurement_review::normalize_global_measurement_reservation_code(&item.code)
}
