use super::*;

use std::collections::BTreeMap;

use crate::federation::metric as federation_metric;

pub fn summarize_adaptive_federation_metrics_for_segment(
    items: &[federation_metric::AdaptiveFederationMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_tiered_token_segments_from_metric(
    items: &[federation_metric::AdaptiveFederationMetric],
) -> Vec<TieredTokenSegment> {
    let mut out: Vec<TieredTokenSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_token_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_tiered_token_segment(item: &TieredTokenSegment) -> f64 {
    federation_metric::tiered_federation_group_weight(2, true)
}
