use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_entry as segment_segment_entry;

pub fn summarize_segment_forecasts_for_cycle(
    items: &[segment_segment_entry::SegmentForecast],
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

pub fn derive_fallback_rule_cycles_from_forecast(
    items: &[segment_segment_entry::SegmentForecast],
) -> Vec<FallbackRuleCycle> {
    let mut out: Vec<FallbackRuleCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_rule_cycle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_fallback_rule_cycle(item: &FallbackRuleCycle) -> f64 {
    segment_segment_entry::batched_segment_forecast_weight(2, true)
}
