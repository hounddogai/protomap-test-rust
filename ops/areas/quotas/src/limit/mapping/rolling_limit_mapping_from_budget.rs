use super::*;

use std::collections::BTreeMap;

use crate::grant::budget as grant_budget;

pub fn summarize_grant_budgets_for_mapping(items: &[grant_budget::GrantBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_limit_mappings_from_budget(items: &[grant_budget::GrantBudget]) -> Vec<RollingLimitMapping> {
    let mut out: Vec<RollingLimitMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_limit_mapping(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rolling_limit_mapping(item: &RollingLimitMapping) -> String {
    grant_budget::rolling_grant_summary_bucket(item.effective_exposure())
}
