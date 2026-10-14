use super::*;

use std::collections::BTreeMap;

use crate::silence::profile as silence_profile;

pub fn summarize_primary_silence_profiles_for_estimate(
    items: &[silence_profile::PrimarySilenceProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_seasonal_budget_burn_estimates_from_profile(
    items: &[silence_profile::PrimarySilenceProfile],
) -> Vec<SeasonalBudgetBurnEstimate> {
    let mut out: Vec<SeasonalBudgetBurnEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_budget_burn_estimate(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_seasonal_budget_burn_estimate(item: &SeasonalBudgetBurnEstimate) -> String {
    silence_profile::classify_primary_silence_profile(item.adjusted_load())
}
