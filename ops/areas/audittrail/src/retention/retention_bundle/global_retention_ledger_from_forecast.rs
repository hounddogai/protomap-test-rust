use super::*;

use std::collections::BTreeMap;

use crate::retention::adaptive_retention_metric as retention_adaptive_retention_metric;

pub fn summarize_fallback_retention_forecasts_for_ledger(
    items: &[retention_adaptive_retention_metric::FallbackRetentionForecast],
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

pub fn derive_global_retention_ledgers_from_forecast(
    items: &[retention_adaptive_retention_metric::FallbackRetentionForecast],
) -> Vec<GlobalRetentionLedger> {
    let mut out: Vec<GlobalRetentionLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_global_retention_ledger(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_global_retention_ledger(item: &GlobalRetentionLedger) -> String {
    retention_adaptive_retention_metric::classify_adaptive_retention_metric(item.weighted_exposure())
}
