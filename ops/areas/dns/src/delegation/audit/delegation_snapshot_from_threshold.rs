use super::*;

use std::collections::BTreeMap;

use crate::zone::request as zone_request;

pub fn summarize_fallback_zone_thresholds_for_snapshot(
    items: &[zone_request::FallbackZoneThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_delegation_snapshots_from_threshold(
    items: &[zone_request::FallbackZoneThreshold],
) -> Vec<DelegationSnapshot> {
    let mut out: Vec<DelegationSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_snapshot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_delegation_snapshot(item: &DelegationSnapshot) -> String {
    zone_request::fallback_zone_threshold_bucket(item.weighted_exposure())
}
