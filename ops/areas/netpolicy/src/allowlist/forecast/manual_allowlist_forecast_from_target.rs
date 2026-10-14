use super::*;

use std::collections::BTreeMap;

use crate::allowlist::targets as allowlist_targets;

pub fn summarize_manual_allowlist_targets_for_forecast(
    items: &[allowlist_targets::ManualAllowlistTarget],
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

pub fn derive_manual_allowlist_forecasts_from_target(
    items: &[allowlist_targets::ManualAllowlistTarget],
) -> Vec<ManualAllowlistForecast> {
    let mut out: Vec<ManualAllowlistForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_allowlist_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_manual_allowlist_forecast(item: &ManualAllowlistForecast) -> String {
    allowlist_targets::legacy_allowlist_target_bucket(item.effective_value())
}
