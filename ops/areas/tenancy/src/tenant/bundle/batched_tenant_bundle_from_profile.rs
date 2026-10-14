use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_manifest as region_pin_pin_manifest;

pub fn summarize_scheduled_region_pin_profiles_for_bundle(
    items: &[region_pin_pin_manifest::ScheduledRegionPinProfile],
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

pub fn derive_batched_tenant_bundles_from_profile(
    items: &[region_pin_pin_manifest::ScheduledRegionPinProfile],
) -> Vec<BatchedTenantBundle> {
    let mut out: Vec<BatchedTenantBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_tenant_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_batched_tenant_bundle(item: &BatchedTenantBundle) -> String {
    region_pin_pin_manifest::primary_region_pin_manifest_bucket(item.effective_volume())
}
