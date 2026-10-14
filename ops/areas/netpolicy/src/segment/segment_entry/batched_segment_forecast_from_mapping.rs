use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_bundle as allowlist_allowlist_bundle;

pub fn summarize_allowlist_mappings_for_forecast(
    items: &[allowlist_allowlist_bundle::AllowlistMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_batched_segment_forecasts_from_mapping(
    items: &[allowlist_allowlist_bundle::AllowlistMapping],
) -> Vec<BatchedSegmentForecast> {
    let mut out: Vec<BatchedSegmentForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_segment_forecast(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_batched_segment_forecast(item: &BatchedSegmentForecast) -> String {
    allowlist_allowlist_bundle::normalize_legacy_allowlist_bundle_code(&item.code)
}
