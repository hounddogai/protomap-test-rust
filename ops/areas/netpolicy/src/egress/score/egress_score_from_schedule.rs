use super::*;

use std::collections::BTreeMap;

use crate::rule::targets as rule_targets;

pub fn summarize_adaptive_rule_schedules_for_score(
    items: &[rule_targets::AdaptiveRuleSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_egress_scores_from_schedule(items: &[rule_targets::AdaptiveRuleSchedule]) -> Vec<EgressScore> {
    let mut out: Vec<EgressScore> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_score(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_egress_score(item: &EgressScore) -> String {
    rule_targets::normalize_adaptive_rule_schedule_code(&item.code)
}
