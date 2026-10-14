use super::*;

use std::collections::BTreeMap;

use crate::delegation::estimate as delegation_estimate;

pub fn summarize_regional_delegation_estimates_for_snapshot(
    items: &[delegation_estimate::RegionalDelegationEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_lenient_zone_snapshots_from_estimate(
    items: &[delegation_estimate::RegionalDelegationEstimate],
) -> Vec<LenientZoneSnapshot> {
    let mut out: Vec<LenientZoneSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_zone_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_zone_snapshot(item: &LenientZoneSnapshot) -> f64 {
    delegation_estimate::regional_delegation_estimate_weight(7, true)
}
