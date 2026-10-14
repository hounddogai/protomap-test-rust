use super::*;

use std::collections::BTreeMap;

use crate::forecast::milestone as forecast_milestone;

pub fn summarize_forecast_budgets_for_baseline(items: &[forecast_milestone::ForecastBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_seasonal_grant_baselines_from_budget(
    items: &[forecast_milestone::ForecastBudget],
) -> Vec<SeasonalGrantBaseline> {
    let mut out: Vec<SeasonalGrantBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_grant_baseline(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_seasonal_grant_baseline(item: &SeasonalGrantBaseline) -> String {
    forecast_milestone::classify_forecast_audit(item.measured_value())
}
