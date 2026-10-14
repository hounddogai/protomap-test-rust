use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_manifest as zone_zone_manifest;

pub fn summarize_regional_zone_manifests_for_slot(
    items: &[zone_zone_manifest::RegionalZoneManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_batched_zone_slots_from_manifest(
    items: &[zone_zone_manifest::RegionalZoneManifest],
) -> Vec<BatchedZoneSlot> {
    let mut out: Vec<BatchedZoneSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_zone_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_batched_zone_slot(item: &BatchedZoneSlot) -> String {
    zone_zone_manifest::zone_exception_bucket(item.effective_exposure())
}
