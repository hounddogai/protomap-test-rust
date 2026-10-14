use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_budget as timeline_timeline_budget;

pub fn summarize_rolling_timeline_budgets_for_rule(
    items: &[timeline_timeline_budget::RollingTimelineBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_global_postmortem_rules_from_budget(
    items: &[timeline_timeline_budget::RollingTimelineBudget],
) -> Vec<GlobalPostmortemRule> {
    let mut out: Vec<GlobalPostmortemRule> = Vec::new();
    for item in items.iter() {
        let derived = new_global_postmortem_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_global_postmortem_rule(item: &GlobalPostmortemRule) -> String {
    timeline_timeline_budget::tiered_timeline_notice_bucket(item.projected_total())
}
