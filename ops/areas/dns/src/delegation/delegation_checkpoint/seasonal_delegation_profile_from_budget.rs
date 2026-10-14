use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_estimate as delegation_delegation_estimate;

pub fn summarize_strict_delegation_budgets_for_profile(
    items: &[delegation_delegation_estimate::StrictDelegationBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_seasonal_delegation_profiles_from_budget(
    items: &[delegation_delegation_estimate::StrictDelegationBudget],
) -> Vec<SeasonalDelegationProfile> {
    let mut out: Vec<SeasonalDelegationProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_delegation_profile(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_delegation_profile(item: &SeasonalDelegationProfile) -> String {
    delegation_delegation_estimate::regional_delegation_template_bucket(item.weighted_exposure())
}
