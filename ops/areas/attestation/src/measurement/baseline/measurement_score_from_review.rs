use super::*;

use std::collections::BTreeMap;

use crate::measurement::review as measurement_review;

pub fn summarize_measurement_reviews_for_score(
    items: &[measurement_review::MeasurementReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_measurement_scores_from_review(items: &[measurement_review::MeasurementReview]) -> Vec<MeasurementScore> {
    let mut out: Vec<MeasurementScore> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_measurement_score_extra(item: &MeasurementScore) -> String {
    measurement_review::normalize_measurement_budget_code(&item.code)
}
