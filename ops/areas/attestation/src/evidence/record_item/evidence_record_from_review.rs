use super::*;

use std::collections::BTreeMap;

use crate::measurement::review as measurement_review;

pub fn summarize_measurement_reviews_for_record(
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

pub fn derive_evidence_records_from_review(items: &[measurement_review::MeasurementReview]) -> Vec<EvidenceRecord> {
    let mut out: Vec<EvidenceRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_record(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_evidence_record(item: &EvidenceRecord) -> String {
    measurement_review::normalize_measurement_budget_code(&item.code)
}
