use super::*;

use std::collections::BTreeMap;

use crate::isolation::isolation_exception as isolation_isolation_exception;

pub fn summarize_strict_isolation_forecasts_for_metric(
    items: &[isolation_isolation_exception::StrictIsolationForecast],
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

pub fn derive_pending_region_pin_metrics_from_forecast(
    items: &[isolation_isolation_exception::StrictIsolationForecast],
) -> Vec<PendingRegionPinMetric> {
    let mut out: Vec<PendingRegionPinMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_region_pin_metric(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_pending_region_pin_metric(item: &PendingRegionPinMetric) -> String {
    isolation_isolation_exception::classify_default_isolation_exception(item.weighted_value())
}
