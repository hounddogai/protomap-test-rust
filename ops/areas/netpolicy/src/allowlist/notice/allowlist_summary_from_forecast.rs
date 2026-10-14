use super::*;

use std::collections::BTreeMap;

use crate::allowlist::forecast as allowlist_forecast;

pub fn summarize_manual_allowlist_forecasts_for_summary(
    items: &[allowlist_forecast::ManualAllowlistForecast],
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

pub fn derive_allowlist_summaries_from_forecast(
    items: &[allowlist_forecast::ManualAllowlistForecast],
) -> Vec<AllowlistSummary> {
    let mut out: Vec<AllowlistSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_allowlist_summary(item: &AllowlistSummary) -> f64 {
    allowlist_forecast::allowlist_budget_weight(8, true)
}
