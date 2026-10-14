use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_manifest as region_pin_pin_manifest;

pub fn summarize_primary_region_pin_manifests_for_draft(
    items: &[region_pin_pin_manifest::PrimaryRegionPinManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_region_pin_drafts_from_manifest(
    items: &[region_pin_pin_manifest::PrimaryRegionPinManifest],
) -> Vec<RegionPinDraft> {
    let mut out: Vec<RegionPinDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_draft(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_region_pin_draft(item: &RegionPinDraft) -> String {
    region_pin_pin_manifest::primary_region_pin_manifest_bucket(item.weighted_value())
}
