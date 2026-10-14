use super::*;

use std::collections::BTreeMap;

use crate::retention::review as retention_review;

pub fn summarize_weighted_retention_forecasts_for_metric(
    items: &[retention_review::WeightedRetentionForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_primary_retention_metrics_from_forecast(
    items: &[retention_review::WeightedRetentionForecast],
) -> Vec<PrimaryRetentionMetric> {
    let mut out: Vec<PrimaryRetentionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_retention_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_primary_retention_metric(item: &PrimaryRetentionMetric) -> f64 {
    retention_review::pending_retention_budget_weight(3, true)
}
