use super::*;

use std::collections::BTreeMap;

use crate::measurement::quota as measurement_quota;

pub fn summarize_manual_measurement_batches_for_summary(
    items: &[measurement_quota::ManualMeasurementBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_nonce_summaries_from_batch(items: &[measurement_quota::ManualMeasurementBatch]) -> Vec<NonceSummary> {
    let mut out: Vec<NonceSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_summary(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_nonce_summary(item: &NonceSummary) -> String {
    measurement_quota::classify_batched_measurement_quota(item.adjusted_load())
}
