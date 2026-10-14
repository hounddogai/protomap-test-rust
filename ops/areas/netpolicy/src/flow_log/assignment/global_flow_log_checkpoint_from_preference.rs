use super::*;

use std::collections::BTreeMap;

use crate::rule::review as rule_review;

pub fn summarize_rule_preferences_for_checkpoint(items: &[rule_review::RulePreference]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_flow_log_checkpoints_from_preference(
    items: &[rule_review::RulePreference],
) -> Vec<GlobalFlowLogCheckpoint> {
    let mut out: Vec<GlobalFlowLogCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_global_flow_log_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_global_flow_log_checkpoint(item: &GlobalFlowLogCheckpoint) -> f64 {
    rule_review::regional_rule_review_weight(2, true)
}
