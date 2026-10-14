use super::*;

use std::collections::BTreeMap;

use crate::silence::score as silence_score;

pub fn summarize_rolling_silence_scores_for_forecast(
    items: &[silence_score::RollingSilenceScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_weighted_budget_burn_forecasts_from_score(
    items: &[silence_score::RollingSilenceScore],
) -> Vec<WeightedBudgetBurnForecast> {
    let mut out: Vec<WeightedBudgetBurnForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_budget_burn_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_weighted_budget_burn_forecast(item: &WeightedBudgetBurnForecast) -> f64 {
    silence_score::rolling_silence_score_weight(9, true)
}
