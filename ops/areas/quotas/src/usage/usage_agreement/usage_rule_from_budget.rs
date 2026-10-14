use super::*;

use std::collections::BTreeMap;

use crate::grant::budget as grant_budget;

pub fn summarize_grant_budgets_for_rule(items: &[grant_budget::GrantBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_usage_rules_from_budget(items: &[grant_budget::GrantBudget]) -> Vec<UsageRule> {
    let mut out: Vec<UsageRule> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_rule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_usage_rule(item: &UsageRule) -> String {
    grant_budget::rolling_grant_summary_bucket(item.net_total())
}
