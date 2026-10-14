use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_entry as segment_segment_entry;

pub fn summarize_batched_segment_forecasts_for_record(
    items: &[segment_segment_entry::BatchedSegmentForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_primary_flow_log_records_from_forecast(
    items: &[segment_segment_entry::BatchedSegmentForecast],
) -> Vec<PrimaryFlowLogRecord> {
    let mut out: Vec<PrimaryFlowLogRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_flow_log_record(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_primary_flow_log_record(item: &PrimaryFlowLogRecord) -> String {
    segment_segment_entry::batched_segment_forecast_bucket(item.effective_load())
}
