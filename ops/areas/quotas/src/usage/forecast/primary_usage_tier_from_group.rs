use super::*;

use std::collections::BTreeMap;

use crate::forecast::tier as forecast_tier;

pub fn summarize_scheduled_forecast_groups_for_tier(
    items: &[forecast_tier::ScheduledForecastGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_primary_usage_tiers_from_group(items: &[forecast_tier::ScheduledForecastGroup]) -> Vec<PrimaryUsageTier> {
    let mut out: Vec<PrimaryUsageTier> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_usage_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_primary_usage_tier(item: &PrimaryUsageTier) -> f64 {
    forecast_tier::scheduled_forecast_group_weight(1, true)
}
