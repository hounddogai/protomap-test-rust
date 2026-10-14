use super::*;

use std::collections::BTreeMap;

use crate::tenant::ledger as tenant_ledger;

pub fn summarize_tenant_allocations_for_manifest(items: &[tenant_ledger::TenantAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_region_pin_manifests_from_allocation(
    items: &[tenant_ledger::TenantAllocation],
) -> Vec<RegionPinManifest> {
    let mut out: Vec<RegionPinManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_manifest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_region_pin_manifest(item: &RegionPinManifest) -> String {
    tenant_ledger::classify_tenant_checkpoint(item.projected_load())
}
