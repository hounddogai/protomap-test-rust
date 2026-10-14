use super::*;

use std::collections::BTreeMap;

use crate::grant::template as grant_template;

pub fn summarize_lenient_grant_budgets_for_milestone(
    items: &[grant_template::LenientGrantBudget],
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

pub fn derive_batched_forecast_milestones_from_budget(
    items: &[grant_template::LenientGrantBudget],
) -> Vec<BatchedForecastMilestone> {
    let mut out: Vec<BatchedForecastMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_forecast_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_batched_forecast_milestone(item: &BatchedForecastMilestone) -> String {
    grant_template::grant_slot_bucket(item.measured_value())
}
