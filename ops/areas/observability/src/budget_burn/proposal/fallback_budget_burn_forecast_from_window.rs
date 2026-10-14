use super::*;

use std::collections::BTreeMap;

use crate::silence::targets as silence_targets;

pub fn summarize_legacy_silence_windows_for_forecast(
    items: &[silence_targets::LegacySilenceWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_fallback_budget_burn_forecasts_from_window(
    items: &[silence_targets::LegacySilenceWindow],
) -> Vec<FallbackBudgetBurnForecast> {
    let mut out: Vec<FallbackBudgetBurnForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_budget_burn_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_fallback_budget_burn_forecast(item: &FallbackBudgetBurnForecast) -> String {
    silence_targets::normalize_legacy_silence_window_code(&item.code)
}
