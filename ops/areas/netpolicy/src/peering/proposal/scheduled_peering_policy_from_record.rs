use super::*;

use std::collections::BTreeMap;

use crate::rule::digest as rule_digest;

pub fn summarize_rule_records_for_policy(items: &[rule_digest::RuleRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_peering_policies_from_record(items: &[rule_digest::RuleRecord]) -> Vec<ScheduledPeeringPolicy> {
    let mut out: Vec<ScheduledPeeringPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_peering_policy(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_peering_policy(item: &ScheduledPeeringPolicy) -> String {
    rule_digest::normalize_regional_rule_digest_code(&item.code)
}
