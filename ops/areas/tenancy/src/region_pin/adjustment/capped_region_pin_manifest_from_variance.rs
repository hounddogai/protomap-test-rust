use super::*;

use std::collections::BTreeMap;

use crate::suspension::assignment as suspension_assignment;

pub fn summarize_suspension_variances_for_manifest(
    items: &[suspension_assignment::SuspensionVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_capped_region_pin_manifests_from_variance(
    items: &[suspension_assignment::SuspensionVariance],
) -> Vec<CappedRegionPinManifest> {
    let mut out: Vec<CappedRegionPinManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_region_pin_manifest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_capped_region_pin_manifest(item: &CappedRegionPinManifest) -> String {
    suspension_assignment::classify_lenient_suspension_assignment(item.weighted_volume())
}
