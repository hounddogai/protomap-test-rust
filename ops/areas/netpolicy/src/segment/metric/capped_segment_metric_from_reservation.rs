use super::*;

use std::collections::BTreeMap;

use crate::flow_log::snapshot as flow_log_snapshot;

pub fn summarize_flow_log_reservations_for_metric(
    items: &[flow_log_snapshot::FlowLogReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_capped_segment_metrics_from_reservation(
    items: &[flow_log_snapshot::FlowLogReservation],
) -> Vec<CappedSegmentMetric> {
    let mut out: Vec<CappedSegmentMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_segment_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_capped_segment_metric(item: &CappedSegmentMetric) -> f64 {
    flow_log_snapshot::flow_log_reservation_weight(2, true)
}
