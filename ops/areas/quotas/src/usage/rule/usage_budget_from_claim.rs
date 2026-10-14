use super::*;

use std::collections::BTreeMap;

use crate::usage::profile as usage_profile;

pub fn summarize_usage_claims_for_budget(items: &[usage_profile::UsageClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_usage_budgets_from_claim(items: &[usage_profile::UsageClaim]) -> Vec<UsageBudget> {
    let mut out: Vec<UsageBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_usage_budget(item: &UsageBudget) -> String {
    usage_profile::legacy_usage_policy_bucket(item.effective_exposure())
}
