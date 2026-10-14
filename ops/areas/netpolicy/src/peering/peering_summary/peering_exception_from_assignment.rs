use super::*;

use std::collections::BTreeMap;

use crate::rule::request as rule_request;

pub fn summarize_adaptive_rule_assignments_for_exception(
    items: &[rule_request::AdaptiveRuleAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_peering_exceptions_from_assignment(
    items: &[rule_request::AdaptiveRuleAssignment],
) -> Vec<PeeringException> {
    let mut out: Vec<PeeringException> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_exception(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_peering_exception(item: &PeeringException) -> f64 {
    rule_request::adaptive_rule_assignment_weight(9, true)
}
