use super::*;

use std::collections::BTreeMap;

use crate::flow_log::score as flow_log_score;

pub fn summarize_flow_log_digests_for_snapshot(items: &[flow_log_score::FlowLogDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_weighted_rule_snapshots_from_digest(
    items: &[flow_log_score::FlowLogDigest],
) -> Vec<WeightedRuleSnapshot> {
    let mut out: Vec<WeightedRuleSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_weighted_rule_snapshot(item: &WeightedRuleSnapshot) -> String {
    flow_log_score::classify_flow_log_score(item.weighted_value())
}
