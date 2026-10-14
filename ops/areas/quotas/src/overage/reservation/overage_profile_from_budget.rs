use super::*;

use std::collections::BTreeMap;

use crate::grant::budget as grant_budget;

pub fn summarize_grant_budgets_for_profile(items: &[grant_budget::GrantBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_overage_profiles_from_budget(items: &[grant_budget::GrantBudget]) -> Vec<OverageProfile> {
    let mut out: Vec<OverageProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_profile(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_overage_profile(item: &OverageProfile) -> String {
    grant_budget::normalize_rolling_grant_summary_code(&item.code)
}
