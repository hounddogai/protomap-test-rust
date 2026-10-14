use super::*;

use std::collections::BTreeMap;

use crate::segment::milestone as segment_milestone;

pub fn summarize_scheduled_segment_baselines_for_mapping(
    items: &[segment_milestone::ScheduledSegmentBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_weighted_flow_log_mappings_from_baseline(
    items: &[segment_milestone::ScheduledSegmentBaseline],
) -> Vec<WeightedFlowLogMapping> {
    let mut out: Vec<WeightedFlowLogMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_flow_log_mapping(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_weighted_flow_log_mapping(item: &WeightedFlowLogMapping) -> String {
    segment_milestone::normalize_scheduled_segment_baseline_code(&item.code)
}
