use super::*;

use std::collections::BTreeMap;

use crate::usage::entry as usage_entry;

pub fn summarize_adaptive_usage_segments_for_milestone(
    items: &[usage_entry::AdaptiveUsageSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_batched_forecast_milestones_from_segment(
    items: &[usage_entry::AdaptiveUsageSegment],
) -> Vec<BatchedForecastMilestone> {
    let mut out: Vec<BatchedForecastMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_forecast_milestone(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_batched_forecast_milestone(item: &BatchedForecastMilestone) -> String {
    usage_entry::normalize_usage_entry_code(&item.code)
}
