use super::*;

use std::collections::BTreeMap;

use crate::usage::forecast as usage_forecast;

pub fn summarize_primary_usage_tiers_for_milestone(
    items: &[usage_forecast::PrimaryUsageTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_strict_forecast_milestones_from_tier(
    items: &[usage_forecast::PrimaryUsageTier],
) -> Vec<StrictForecastMilestone> {
    let mut out: Vec<StrictForecastMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_forecast_milestone(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_strict_forecast_milestone(item: &StrictForecastMilestone) -> String {
    usage_forecast::classify_adaptive_usage_override(item.measured_value())
}
