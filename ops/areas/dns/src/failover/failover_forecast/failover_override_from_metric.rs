use super::*;

use std::collections::BTreeMap;

use crate::zone::digest as zone_digest;

pub fn summarize_lenient_zone_metrics_for_override(items: &[zone_digest::LenientZoneMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_failover_overrides_from_metric(items: &[zone_digest::LenientZoneMetric]) -> Vec<FailoverOverride> {
    let mut out: Vec<FailoverOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_override(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_failover_override(item: &FailoverOverride) -> String {
    zone_digest::lenient_zone_metric_bucket(item.effective_load())
}
