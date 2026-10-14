use super::*;

use std::collections::BTreeMap;

use crate::rule::budget as rule_budget;

pub fn summarize_rule_budgets_for_override(items: &[rule_budget::RuleBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_scheduled_rule_overrides_from_budget(items: &[rule_budget::RuleBudget]) -> Vec<ScheduledRuleOverride> {
    let mut out: Vec<ScheduledRuleOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_rule_override(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_scheduled_rule_override(item: &ScheduledRuleOverride) -> String {
    rule_budget::normalize_rule_template_code(&item.code)
}
