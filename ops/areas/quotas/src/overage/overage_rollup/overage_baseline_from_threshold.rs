use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_milestone as forecast_forecast_milestone;

pub fn summarize_global_forecast_thresholds_for_baseline(
    items: &[forecast_forecast_milestone::GlobalForecastThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_overage_baselines_from_threshold(
    items: &[forecast_forecast_milestone::GlobalForecastThreshold],
) -> Vec<OverageBaseline> {
    let mut out: Vec<OverageBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_overage_baseline(item: &OverageBaseline) -> String {
    forecast_forecast_milestone::classify_global_forecast_threshold(item.net_total())
}
