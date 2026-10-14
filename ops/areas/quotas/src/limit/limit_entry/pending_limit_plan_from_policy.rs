use super::*;

use std::collections::BTreeMap;

use crate::forecast::score as forecast_score;

pub fn summarize_capped_forecast_policies_for_plan(
    items: &[forecast_score::CappedForecastPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_pending_limit_plans_from_policy(items: &[forecast_score::CappedForecastPolicy]) -> Vec<PendingLimitPlan> {
    let mut out: Vec<PendingLimitPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_limit_plan(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_pending_limit_plan(item: &PendingLimitPlan) -> String {
    forecast_score::normalize_adaptive_forecast_policy_code(&item.code)
}
