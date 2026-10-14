use super::*;

use std::collections::BTreeMap;

use crate::measurement::measurement_mapping as measurement_measurement_mapping;

pub fn summarize_default_measurement_mappings_for_review(
    items: &[measurement_measurement_mapping::DefaultMeasurementMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_scheduled_verifier_reviews_from_mapping(
    items: &[measurement_measurement_mapping::DefaultMeasurementMapping],
) -> Vec<ScheduledVerifierReview> {
    let mut out: Vec<ScheduledVerifierReview> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_verifier_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_verifier_review(item: &ScheduledVerifierReview) -> f64 {
    measurement_measurement_mapping::capped_measurement_quota_weight(9, true)
}
