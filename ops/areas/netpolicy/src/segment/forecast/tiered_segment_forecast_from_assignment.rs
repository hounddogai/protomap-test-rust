use super::*;

use std::collections::BTreeMap;

use crate::rule::request as rule_request;

pub fn summarize_adaptive_rule_assignments_for_forecast(
    items: &[rule_request::AdaptiveRuleAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_segment_forecasts_from_assignment(
    items: &[rule_request::AdaptiveRuleAssignment],
) -> Vec<TieredSegmentForecast> {
    let mut out: Vec<TieredSegmentForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_segment_forecast(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_tiered_segment_forecast(item: &TieredSegmentForecast) -> String {
    rule_request::strict_rule_request_bucket(item.adjusted_load())
}
