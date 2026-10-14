use super::*;

use std::collections::BTreeMap;

use crate::nonce::snapshot as nonce_snapshot;

pub fn summarize_regional_nonce_segments_for_estimate(
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

pub fn derive_fallback_nonce_estimates_from_segment(
    items: &[nonce_snapshot::RegionalNonceSegment],
) -> Vec<FallbackNonceEstimate> {
    let mut out: Vec<FallbackNonceEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_nonce_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_nonce_estimate(item: &FallbackNonceEstimate) -> String {
    nonce_snapshot::normalize_nonce_snapshot_code(&item.code)
}
