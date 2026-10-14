use super::*;

use std::collections::BTreeMap;

use crate::health_check::check_rule as health_check_check_rule;

pub fn summarize_default_health_check_preferences_for_entry(
    items: &[health_check_check_rule::DefaultHealthCheckPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_scheduled_dns_record_entries_from_preference(
    items: &[health_check_check_rule::DefaultHealthCheckPreference],
) -> Vec<ScheduledDnsRecordEntry> {
    let mut out: Vec<ScheduledDnsRecordEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_dns_record_entry(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_scheduled_dns_record_entry(item: &ScheduledDnsRecordEntry) -> String {
    health_check_check_rule::pending_health_check_rule_bucket(item.weighted_total())
}
