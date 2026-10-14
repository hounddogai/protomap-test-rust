use super::*;

use std::collections::BTreeMap;

use crate::dns_record::rule as dns_record_rule;

pub fn summarize_dns_record_cycles_for_preference(items: &[dns_record_rule::DnsRecordCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_scheduled_health_check_preferences_from_cycle(
    items: &[dns_record_rule::DnsRecordCycle],
) -> Vec<ScheduledHealthCheckPreference> {
    let mut out: Vec<ScheduledHealthCheckPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_health_check_preference(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_scheduled_health_check_preference(item: &ScheduledHealthCheckPreference) -> String {
    dns_record_rule::classify_global_dns_record_entry(item.weighted_value())
}
