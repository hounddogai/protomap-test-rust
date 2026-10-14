use super::*;

use std::collections::BTreeMap;

use crate::verifier::quota as verifier_quota;

pub fn summarize_verifier_quotas_for_review(items: &[verifier_quota::VerifierQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_measurement_reviews_from_quota(items: &[verifier_quota::VerifierQuota]) -> Vec<MeasurementReview> {
    let mut out: Vec<MeasurementReview> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_review(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_measurement_review(item: &MeasurementReview) -> String {
    verifier_quota::normalize_verifier_quota_code(&item.code)
}
