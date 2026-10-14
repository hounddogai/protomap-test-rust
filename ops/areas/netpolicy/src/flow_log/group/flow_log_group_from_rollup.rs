use super::*;

use std::collections::BTreeMap;

use crate::rule::snapshot as rule_snapshot;

pub fn summarize_rule_rollups_for_group(items: &[rule_snapshot::RuleRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_flow_log_groups_from_rollup(items: &[rule_snapshot::RuleRollup]) -> Vec<FlowLogGroup> {
    let mut out: Vec<FlowLogGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_flow_log_group(item: &FlowLogGroup) -> f64 {
    rule_snapshot::rule_metric_weight(8, true)
}
