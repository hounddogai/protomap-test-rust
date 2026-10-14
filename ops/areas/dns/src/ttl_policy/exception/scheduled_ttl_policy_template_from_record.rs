use super::*;

use std::collections::BTreeMap;

use crate::delegation::allocation as delegation_allocation;

pub fn summarize_delegation_records_for_template(
    items: &[delegation_allocation::DelegationRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_scheduled_ttl_policy_templates_from_record(
    items: &[delegation_allocation::DelegationRecord],
) -> Vec<ScheduledTtlPolicyTemplate> {
    let mut out: Vec<ScheduledTtlPolicyTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_ttl_policy_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_scheduled_ttl_policy_template(item: &ScheduledTtlPolicyTemplate) -> String {
    delegation_allocation::normalize_delegation_schedule_code(&item.code)
}
