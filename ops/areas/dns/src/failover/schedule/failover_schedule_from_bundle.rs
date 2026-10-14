use super::*;

use std::collections::BTreeMap;

use crate::zone::bundle as zone_bundle;

pub fn summarize_zone_bundles_for_schedule(items: &[zone_bundle::ZoneBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_failover_schedules_from_bundle(items: &[zone_bundle::ZoneBundle]) -> Vec<FailoverSchedule> {
    let mut out: Vec<FailoverSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_failover_schedule(item: &FailoverSchedule) -> String {
    zone_bundle::legacy_zone_override_bucket(item.weighted_volume())
}
