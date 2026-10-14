use super::*;

use std::collections::BTreeMap;

use crate::action_item::policy as action_item_policy;

pub fn summarize_action_item_policies_for_budget(
    items: &[action_item_policy::ActionItemPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_timeline_budgets_from_policy(items: &[action_item_policy::ActionItemPolicy]) -> Vec<TimelineBudget> {
    let mut out: Vec<TimelineBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_timeline_budget(item: &TimelineBudget) -> String {
    action_item_policy::classify_action_item_draft(item.net_volume())
}
