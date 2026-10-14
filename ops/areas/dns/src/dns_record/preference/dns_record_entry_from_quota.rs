use super::*;

use std::collections::BTreeMap;

use crate::zone::quota as zone_quota;

pub fn summarize_zone_quotas_for_entry(items: &[zone_quota::ZoneQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_dns_record_entries_from_quota(items: &[zone_quota::ZoneQuota]) -> Vec<DnsRecordEntry> {
    let mut out: Vec<DnsRecordEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_entry(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_dns_record_entry(item: &DnsRecordEntry) -> f64 {
    zone_quota::zone_quota_weight(7, true)
}
