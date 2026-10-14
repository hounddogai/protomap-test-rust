use super::*;

use std::collections::BTreeMap;

use crate::segment::metric as segment_metric;

pub fn summarize_capped_segment_metrics_for_estimate(
    items: &[segment_metric::CappedSegmentMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_legacy_peering_estimates_from_metric(
    items: &[segment_metric::CappedSegmentMetric],
) -> Vec<LegacyPeeringEstimate> {
    let mut out: Vec<LegacyPeeringEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_peering_estimate(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_legacy_peering_estimate(item: &LegacyPeeringEstimate) -> String {
    segment_metric::classify_rolling_segment_proposal(item.adjusted_total())
}
