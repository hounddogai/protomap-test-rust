use super::*;

use std::collections::BTreeMap;

use crate::rule::exception as rule_exception;

pub fn summarize_rule_requests_for_group(items: &[rule_exception::RuleRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rule_groups_from_request(items: &[rule_exception::RuleRequest]) -> Vec<RuleGroup> {
    let mut out: Vec<RuleGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_rule_group(item: &RuleGroup) -> String {
    rule_exception::classify_rule_snapshot(item.adjusted_volume())
}
