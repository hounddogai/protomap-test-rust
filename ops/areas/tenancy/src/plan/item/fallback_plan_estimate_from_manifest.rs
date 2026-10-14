use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_manifest as region_pin_pin_manifest;

pub fn summarize_lenient_region_pin_manifests_for_estimate(
    items: &[region_pin_pin_manifest::LenientRegionPinManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_fallback_plan_estimates_from_manifest(
    items: &[region_pin_pin_manifest::LenientRegionPinManifest],
) -> Vec<FallbackPlanEstimate> {
    let mut out: Vec<FallbackPlanEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_plan_estimate(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_fallback_plan_estimate(item: &FallbackPlanEstimate) -> String {
    region_pin_pin_manifest::normalize_primary_region_pin_manifest_code(&item.code)
}
