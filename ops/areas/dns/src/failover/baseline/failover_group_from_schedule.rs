use super::*;

use std::collections::BTreeMap;

use crate::dns_record::schedule as dns_record_schedule;

pub fn summarize_dns_record_schedules_for_group(
    items: &[dns_record_schedule::DnsRecordSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_failover_groups_from_schedule(items: &[dns_record_schedule::DnsRecordSchedule]) -> Vec<FailoverGroup> {
    let mut out: Vec<FailoverGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_group(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_failover_group(item: &FailoverGroup) -> String {
    dns_record_schedule::normalize_default_dns_record_batch_code(&item.code)
}
