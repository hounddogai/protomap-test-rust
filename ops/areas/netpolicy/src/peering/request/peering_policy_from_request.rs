use super::*;

use std::collections::BTreeMap;

use crate::rule::exception as rule_exception;

pub fn summarize_rule_requests_for_policy(items: &[rule_exception::RuleRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_peering_policies_from_request(items: &[rule_exception::RuleRequest]) -> Vec<PeeringPolicy> {
    let mut out: Vec<PeeringPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_peering_policy(item: &PeeringPolicy) -> String {
    rule_exception::rule_request_bucket(item.weighted_load())
}
