use super::*;

use std::collections::BTreeMap;

use crate::measurement::forecast as measurement_forecast;

pub fn summarize_measurement_forecasts_for_segment(
    items: &[measurement_forecast::MeasurementForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_strict_policy_segments_from_forecast(
    items: &[measurement_forecast::MeasurementForecast],
) -> Vec<StrictPolicySegment> {
    let mut out: Vec<StrictPolicySegment> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_policy_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_strict_policy_segment(item: &StrictPolicySegment) -> String {
    measurement_forecast::normalize_measurement_forecast_code(&item.code)
}
