use super::*;

use std::collections::BTreeMap;

use crate::release::slot as release_slot;

pub fn summarize_release_entries_for_forecast(items: &[release_slot::ReleaseEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollout_forecasts_from_entry(items: &[release_slot::ReleaseEntry]) -> Vec<RolloutForecast> {
    let mut out: Vec<RolloutForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_rollout_forecast(item: &RolloutForecast) -> String {
    release_slot::normalize_release_entry_code(&item.code)
}
