use super::*;

use std::collections::BTreeMap;

use crate::flow_log::forecast as flow_log_forecast;

pub fn summarize_flow_log_forecasts_for_review(items: &[flow_log_forecast::FlowLogForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_tiered_segment_reviews_from_forecast(
    items: &[flow_log_forecast::FlowLogForecast],
) -> Vec<TieredSegmentReview> {
    let mut out: Vec<TieredSegmentReview> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_segment_review(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_segment_review(item: &TieredSegmentReview) -> String {
    flow_log_forecast::classify_flow_log_checkpoint(item.adjusted_volume())
}
