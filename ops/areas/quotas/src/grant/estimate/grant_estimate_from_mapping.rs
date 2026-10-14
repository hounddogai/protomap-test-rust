use super::*;

use std::collections::BTreeMap;

use crate::forecast::mapping as forecast_mapping;

pub fn summarize_forecast_mappings_for_estimate(items: &[forecast_mapping::ForecastMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_grant_estimates_from_mapping(items: &[forecast_mapping::ForecastMapping]) -> Vec<GrantEstimate> {
    let mut out: Vec<GrantEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_grant_estimate(item: &GrantEstimate) -> String {
    forecast_mapping::forecast_review_bucket(item.weighted_total())
}
