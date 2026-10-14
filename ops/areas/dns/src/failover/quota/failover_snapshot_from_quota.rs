use super::*;

use std::collections::BTreeMap;

use crate::zone::quota as zone_quota;

pub fn summarize_zone_quotas_for_snapshot(items: &[zone_quota::ZoneQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_failover_snapshots_from_quota(items: &[zone_quota::ZoneQuota]) -> Vec<FailoverSnapshot> {
    let mut out: Vec<FailoverSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_failover_snapshot(item: &FailoverSnapshot) -> String {
    zone_quota::classify_zone_override(item.adjusted_volume())
}
