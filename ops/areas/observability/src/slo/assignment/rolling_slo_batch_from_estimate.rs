use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::burn_preference as budget_burn_burn_preference;

pub fn summarize_seasonal_budget_burn_estimates_for_batch(
    items: &[budget_burn_burn_preference::SeasonalBudgetBurnEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rolling_slo_batches_from_estimate(
    items: &[budget_burn_burn_preference::SeasonalBudgetBurnEstimate],
) -> Vec<RollingSloBatch> {
    let mut out: Vec<RollingSloBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_slo_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_rolling_slo_batch(item: &RollingSloBatch) -> String {
    budget_burn_burn_preference::normalize_fallback_budget_burn_preference_code(&item.code)
}
