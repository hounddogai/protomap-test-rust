use super::*;

use std::collections::BTreeMap;

use crate::forecast::threshold as forecast_threshold;

pub fn summarize_forecast_thresholds_for_segment(
    items: &[forecast_threshold::ForecastThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_default_forecast_segments_from_threshold(
    items: &[forecast_threshold::ForecastThreshold],
) -> Vec<DefaultForecastSegment> {
    let mut out: Vec<DefaultForecastSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_forecast_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_default_forecast_segment(item: &DefaultForecastSegment) -> String {
    forecast_threshold::forecast_rule_bucket(item.effective_exposure())
}
