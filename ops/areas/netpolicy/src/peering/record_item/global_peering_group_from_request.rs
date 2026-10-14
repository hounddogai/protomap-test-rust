use super::*;

use std::collections::BTreeMap;

use crate::rule::targets as rule_targets;

pub fn summarize_lenient_rule_requests_for_group(items: &[rule_targets::LenientRuleRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_global_peering_groups_from_request(
    items: &[rule_targets::LenientRuleRequest],
) -> Vec<GlobalPeeringGroup> {
    let mut out: Vec<GlobalPeeringGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_global_peering_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_global_peering_group(item: &GlobalPeeringGroup) -> String {
    rule_targets::adaptive_rule_schedule_bucket(item.net_value())
}
