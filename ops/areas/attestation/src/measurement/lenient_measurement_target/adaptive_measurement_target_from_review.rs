use super::*;

use std::collections::BTreeMap;

use crate::nonce::preference as nonce_preference;

pub fn summarize_nonce_reviews_for_target(items: &[nonce_preference::NonceReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_adaptive_measurement_targets_from_review(
    items: &[nonce_preference::NonceReview],
) -> Vec<AdaptiveMeasurementTarget> {
    let mut out: Vec<AdaptiveMeasurementTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_measurement_target(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_adaptive_measurement_target(item: &AdaptiveMeasurementTarget) -> String {
    nonce_preference::nonce_review_bucket(item.effective_volume())
}
