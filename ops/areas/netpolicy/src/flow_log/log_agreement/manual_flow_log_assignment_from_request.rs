use super::*;

use std::collections::BTreeMap;

use crate::rule::rule_request as rule_rule_request;

pub fn summarize_adaptive_rule_requests_for_assignment(
    items: &[rule_rule_request::AdaptiveRuleRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_manual_flow_log_assignments_from_request(
    items: &[rule_rule_request::AdaptiveRuleRequest],
) -> Vec<ManualFlowLogAssignment> {
    let mut out: Vec<ManualFlowLogAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_flow_log_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_manual_flow_log_assignment(item: &ManualFlowLogAssignment) -> String {
    rule_rule_request::primary_rule_batch_bucket(item.effective_total())
}
