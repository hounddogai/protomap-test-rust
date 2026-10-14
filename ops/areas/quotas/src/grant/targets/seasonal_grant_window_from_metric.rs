use super::*;

use std::collections::BTreeMap;

use crate::usage::metric as usage_metric;

pub fn summarize_global_usage_metrics_for_window(items: &[usage_metric::GlobalUsageMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_seasonal_grant_windows_from_metric(
    items: &[usage_metric::GlobalUsageMetric],
) -> Vec<SeasonalGrantWindow> {
    let mut out: Vec<SeasonalGrantWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_grant_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_grant_window(item: &SeasonalGrantWindow) -> String {
    usage_metric::global_usage_metric_bucket(item.adjusted_total())
}
