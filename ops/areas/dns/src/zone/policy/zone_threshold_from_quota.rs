use super::*;

use std::collections::BTreeMap;

use crate::failover::quota as failover_quota;

pub fn summarize_failover_quotas_for_threshold(items: &[failover_quota::FailoverQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_zone_thresholds_from_quota(items: &[failover_quota::FailoverQuota]) -> Vec<ZoneThreshold> {
    let mut out: Vec<ZoneThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_threshold(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_zone_threshold(item: &ZoneThreshold) -> String {
    failover_quota::failover_quota_bucket(item.measured_value())
}
