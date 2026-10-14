use super::*;

use std::collections::BTreeMap;

use crate::dns_record::policy as dns_record_policy;

pub fn summarize_dns_record_policies_for_allocation(
    items: &[dns_record_policy::DnsRecordPolicy],
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

pub fn derive_scheduled_failover_allocations_from_policy(
    items: &[dns_record_policy::DnsRecordPolicy],
) -> Vec<ScheduledFailoverAllocation> {
    let mut out: Vec<ScheduledFailoverAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_failover_allocation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_failover_allocation(item: &ScheduledFailoverAllocation) -> f64 {
    dns_record_policy::dns_record_score_weight(8, true)
}
