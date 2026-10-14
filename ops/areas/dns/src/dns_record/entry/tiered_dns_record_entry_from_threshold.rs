use super::*;

use std::collections::BTreeMap;

use crate::health_check::proposal as health_check_proposal;

pub fn summarize_rolling_health_check_thresholds_for_entry(
    items: &[health_check_proposal::RollingHealthCheckThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_tiered_dns_record_entries_from_threshold(
    items: &[health_check_proposal::RollingHealthCheckThreshold],
) -> Vec<TieredDnsRecordEntry> {
    let mut out: Vec<TieredDnsRecordEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_dns_record_entry(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_tiered_dns_record_entry(item: &TieredDnsRecordEntry) -> f64 {
    health_check_proposal::health_check_estimate_weight(9, true)
}
