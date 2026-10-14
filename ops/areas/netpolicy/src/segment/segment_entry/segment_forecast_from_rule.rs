use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_policy as segment_segment_policy;

pub fn summarize_segment_rules_for_forecast(items: &[segment_segment_policy::SegmentRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_segment_forecasts_from_rule(items: &[segment_segment_policy::SegmentRule]) -> Vec<SegmentForecast> {
    let mut out: Vec<SegmentForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_forecast(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_segment_forecast(item: &SegmentForecast) -> f64 {
    segment_segment_policy::segment_allocation_weight(5, true)
}
