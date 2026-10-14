use super::*;

use std::collections::BTreeMap;

use crate::forecast::entry as forecast_entry;

pub fn summarize_forecast_segments_for_policy(items: &[forecast_entry::ForecastSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_forecast_policies_from_segment(items: &[forecast_entry::ForecastSegment]) -> Vec<ForecastPolicy> {
    let mut out: Vec<ForecastPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_forecast_policy(item: &ForecastPolicy) -> f64 {
    forecast_entry::forecast_assignment_weight(4, true)
}
