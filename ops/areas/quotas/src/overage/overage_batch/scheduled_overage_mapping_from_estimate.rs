use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_estimate as forecast_forecast_estimate;

pub fn summarize_lenient_forecast_estimates_for_mapping(
    items: &[forecast_forecast_estimate::LenientForecastEstimate],
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

pub fn derive_scheduled_overage_mappings_from_estimate(
    items: &[forecast_forecast_estimate::LenientForecastEstimate],
) -> Vec<ScheduledOverageMapping> {
    let mut out: Vec<ScheduledOverageMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_overage_mapping(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_scheduled_overage_mapping(item: &ScheduledOverageMapping) -> String {
    forecast_forecast_estimate::classify_lenient_forecast_estimate(item.adjusted_exposure())
}
