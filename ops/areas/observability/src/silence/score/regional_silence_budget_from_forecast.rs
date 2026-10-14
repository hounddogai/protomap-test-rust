use super::*;

use std::collections::BTreeMap;

use crate::slo::forecast as slo_forecast;

pub fn summarize_pending_slo_forecasts_for_budget(items: &[slo_forecast::PendingSloForecast]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_regional_silence_budgets_from_forecast(
    items: &[slo_forecast::PendingSloForecast],
) -> Vec<RegionalSilenceBudget> {
    let mut out: Vec<RegionalSilenceBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_silence_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_regional_silence_budget(item: &RegionalSilenceBudget) -> String {
    slo_forecast::classify_pending_slo_forecast(item.net_total())
}
