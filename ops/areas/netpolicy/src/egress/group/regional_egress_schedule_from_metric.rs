use super::*;

use std::collections::BTreeMap;

use crate::segment::metric as segment_metric;

pub fn summarize_capped_segment_metrics_for_schedule(
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

pub fn derive_regional_egress_schedules_from_metric(
    items: &[segment_metric::CappedSegmentMetric],
) -> Vec<RegionalEgressSchedule> {
    let mut out: Vec<RegionalEgressSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_egress_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_regional_egress_schedule(item: &RegionalEgressSchedule) -> String {
    segment_metric::classify_rolling_segment_proposal(item.measured_value())
}
