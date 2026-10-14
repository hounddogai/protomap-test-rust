use super::*;

use std::collections::BTreeMap;

use crate::nonce::snapshot as nonce_snapshot;

pub fn summarize_regional_nonce_segments_for_score(
    items: &[nonce_snapshot::RegionalNonceSegment],
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

pub fn derive_measurement_scores_from_segment(items: &[nonce_snapshot::RegionalNonceSegment]) -> Vec<MeasurementScore> {
    let mut out: Vec<MeasurementScore> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_score(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_measurement_score(item: &MeasurementScore) -> String {
    nonce_snapshot::classify_regional_nonce_segment(item.net_exposure())
}
