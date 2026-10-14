use super::*;

use std::collections::BTreeMap;

use crate::query::audit as query_audit;

pub fn summarize_query_schedules_for_forecast(items: &[query_audit::QuerySchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_weighted_retention_forecasts_from_schedule(
    items: &[query_audit::QuerySchedule],
) -> Vec<WeightedRetentionForecast> {
    let mut out: Vec<WeightedRetentionForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_retention_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_weighted_retention_forecast(item: &WeightedRetentionForecast) -> f64 {
    query_audit::pending_query_audit_weight(1, true)
}
