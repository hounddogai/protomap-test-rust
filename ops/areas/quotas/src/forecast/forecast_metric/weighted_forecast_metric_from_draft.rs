use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_rollup as overage_overage_rollup;

pub fn summarize_adaptive_overage_drafts_for_metric(
    items: &[overage_overage_rollup::AdaptiveOverageDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_weighted_forecast_metrics_from_draft(
    items: &[overage_overage_rollup::AdaptiveOverageDraft],
) -> Vec<WeightedForecastMetric> {
    let mut out: Vec<WeightedForecastMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_forecast_metric(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_forecast_metric(item: &WeightedForecastMetric) -> String {
    overage_overage_rollup::classify_adaptive_overage_draft(item.weighted_load())
}
